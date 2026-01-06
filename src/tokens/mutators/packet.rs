use libafl_bolts::prelude::{Rand, StdRand};
use libafl::prelude::{MutationResult, Error, HasRand, HasMetadata, HasCorpus, random_corpus_id, Corpus};
use crate::{
    packets::PacketBasedInput,
    tokens::*,
    packets::PacketMutator,
};

pub struct TokenStreamPacketMutator<const M: usize> {
    rand: StdRand,
}

impl<const M: usize> Default for TokenStreamPacketMutator<M> {
    fn default() -> Self {
        Self {
            rand: StdRand::new(),
        }
    }
}

impl<const M: usize, S> PacketMutator<TokenStream, S> for TokenStreamPacketMutator<M>
where
    S: HasRand + HasMetadata + HasCorpus<PacketBasedInput<TokenStream>>,
{
    fn mutate_packet(&mut self, state: &mut S, packet: &mut TokenStream) -> Result<MutationResult, Error> {
        self.rand.set_seed(state.rand_mut().next());
        let stack = state.rand_mut().choose(MUTATOR_STACKS).unwrap();
        let mut mutated = false;
        
        for _ in 0..stack {
            let m = self.rand.between(0, NUM_MUTATORS + 1);
            
            mutated |= if m < NUM_MUTATORS {
                mutate_non_crossover::<_, _, 16, 4>(m, packet, state, &mut self.rand, M)
            } else {
                let idx = random_corpus_id!(state.corpus(), &mut self.rand);
                
                if state.corpus().current().as_ref() == Some(&idx) {
                    continue;
                }
                
                let mut other_testcase = state.corpus().get(idx)?.borrow_mut();
                let other_testcase = other_testcase.load_input(state.corpus())?;
                
                if other_testcase.packets().is_empty() {
                    continue;
                }
                
                let idx = self.rand.between(0, other_testcase.packets().len() - 1);
                let other_packet = &other_testcase.packets()[idx];
                
                if m == NUM_MUTATORS {
                    mutate_crossover_insert(&mut self.rand, packet, other_packet, M)
                } else {
                    mutate_crossover_replace(&mut self.rand, packet, other_packet, M)
                }
            };
        }
        
        if mutated {
            Ok(MutationResult::Mutated)
        } else {
            Ok(MutationResult::Skipped)
        }
    }
}
