use libafl::prelude::{
    Mutator, MutationResult, Error, Tokens, HasCorpus,
    random_corpus_id, Corpus, HasMetadata, HasRand,
};
use libafl_bolts::prelude::{Named, Rand, StdRand, HasLen};
use crate::tokens::{TokenStream, mutators::*};
use std::borrow::Cow;

pub(crate) const MUTATOR_STACKS: [usize; 5] = [
    2,
    4,
    8,
    16,
    32,
];
pub(crate) const NUM_MUTATORS: usize = 17;

#[inline]
pub(crate) fn mutate_non_crossover<R, S, const L: usize, const N: usize>(idx: usize, stream: &mut TokenStream, state: &mut S, rand: &mut R, max_tokens: usize) -> bool
where
    R: Rand,
    S: HasMetadata,
{
    match idx {
        0 => mutate_copy(rand, stream, max_tokens),
        1 => mutate_delete(rand, stream),
        2 => mutate_flip(rand, stream),
        3 => mutate_interesting(rand, stream),
        4 => mutate_random_insert(rand, stream, max_tokens),
        5 => mutate_random_replace(rand, stream),
        6 => mutate_repeat_char::<_, L>(rand, stream),
        7 => mutate_repeat_token::<_, N>(rand, stream, max_tokens),
        8 => mutate_special_insert(rand, stream),
        9 => mutate_special_replace(rand, stream),
        10 => mutate_split(rand, stream, max_tokens),
        11 => mutate_swap_tokens(rand, stream),
        12 => mutate_swap_words(rand, stream),
        13 => mutate_truncate(rand, stream),
        14 => {
            let dict = state.metadata_map().get::<Tokens>();

            if let Some(dict) = dict {
                mutate_dict_insert(rand, stream, dict, max_tokens)
            } else {
                false
            }
        },
        15 => {
            let dict = state.metadata_map().get::<Tokens>();

            if let Some(dict) = dict {
                mutate_dict_replace(rand, stream, dict)
            } else {
                false
            }
        },
        16 => {
            let dict = state.metadata_map().get::<Tokens>();

            if let Some(dict) = dict {
                mutate_swap_constants(rand, stream, dict)
            } else {
                false
            }
        },
        _ => unreachable!(),
    }
}

#[derive(Default)]
pub struct TokenStreamMutator<const M: usize> {
    rand: StdRand,
}

impl<const M: usize> Named for TokenStreamMutator<M> {
    fn name(&self) -> &Cow<'static, str> {
        static NAME: Cow<'static, str> = Cow::Borrowed("TokenStreamMutator");
        &NAME
    }
}

impl<S, const M: usize> Mutator<TokenStream, S> for TokenStreamMutator<M>
where
    S: HasRand + HasMetadata + HasCorpus<TokenStream>,
{
    fn mutate(&mut self, state: &mut S, input: &mut TokenStream) -> Result<MutationResult, Error> {
        self.rand.set_seed(state.rand_mut().next());
        let stack = self.rand.choose(MUTATOR_STACKS).unwrap();
        let mut mutated = false;
        
        for _ in 0..stack {
            let idx = self.rand.between(0, NUM_MUTATORS + 1);
            mutated |= if idx < NUM_MUTATORS {
                mutate_non_crossover::<_, _, 16, 4>(idx, input, state, &mut self.rand, M)
            } else {
                let id = random_corpus_id!(state.corpus(), &mut self.rand);
                
                if state.corpus().current().as_ref() == Some(&id) {
                    continue;
                }
                
                let mut other_testcase = state.corpus().get(id)?.borrow_mut();
                let other_testcase = other_testcase.load_input(state.corpus())?;
                
                if other_testcase.is_empty() {
                    continue;
                }
                
                if idx == NUM_MUTATORS {
                    mutate_crossover_insert(&mut self.rand, input, other_testcase, M)
                } else {
                    mutate_crossover_replace(&mut self.rand, input, other_testcase, M)
                }
            };
        }
        
        if mutated {
            Ok(MutationResult::Mutated)
        } else {
            Ok(MutationResult::Skipped)
        }
    }
    
    fn post_exec(&mut self, _state: &mut S, _new_corpus_id: Option<libafl::prelude::CorpusId>) -> Result<(), Error> {
        Ok(())
    }
}
