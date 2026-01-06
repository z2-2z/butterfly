use libafl_bolts::prelude::{Named, Rand};
use libafl::prelude::{
    Mutator, MutationResult, Error, HasRand,
    HavocMutationsNoCrossoverType, havoc_mutations_no_crossover,
    BytesInput, HasMetadata, HasCorpus, HasMaxSize,
    MutationId, MutatorsTuple,
};
use crate::packets::{PacketBasedInput, Packet};
use std::marker::PhantomData;
use std::borrow::Cow;
use std::num::NonZero;

pub trait PacketMutator<P, S>
where
    P: Packet,
{
    fn mutate_packet(&mut self, state: &mut S, packet: &mut P) -> Result<MutationResult, Error>;
}

pub struct PacketContentMutator<P, S, M>
where
    M: PacketMutator<P, S>,
    P: Packet,
{
    mutator: M,
    phantom: PhantomData<(P, S)>,
}

impl<P, S, M> PacketContentMutator<P, S, M>
where
    M: PacketMutator<P, S>,
    P: Packet,
{
    #[allow(clippy::new_without_default)]
    pub fn new(mutator: M) -> Self {
        Self {
            mutator,
            phantom: PhantomData,
        }
    }
}

impl<P, S, M> Named for PacketContentMutator<P, S, M>
where
    M: PacketMutator<P, S>,
    P: Packet,
{
    fn name(&self) -> &Cow<'static, str> {
        static NAME: Cow<'static, str> = Cow::Borrowed("PacketContentMutator");
        &NAME
    }
}

impl<P, S, M> Mutator<PacketBasedInput<P>, S> for PacketContentMutator<P, S, M>
where
    M: PacketMutator<P, S>,
    P: Packet,
    S: HasRand,
{
    fn mutate(&mut self, state: &mut S, input: &mut PacketBasedInput<P>) -> Result<MutationResult, Error> {
        let len = input.packets().len();
        
        if len == 0 {
            return Ok(MutationResult::Skipped);
        }
        
        let idx = state.rand_mut().between(0, len - 1);
        let packet = &mut input.packets_mut()[idx];
        self.mutator.mutate_packet(state, packet)
    }
    
    fn post_exec(&mut self, _state: &mut S, _new_corpus_id: Option<libafl::prelude::CorpusId>) -> Result<(), Error> {
        Ok(())
    }
}

pub struct PacketHavocMutator {
    mutators: HavocMutationsNoCrossoverType,
}

impl Default for PacketHavocMutator {
    fn default() -> Self {
        Self {
            mutators: havoc_mutations_no_crossover(),
        }
    }
}

impl<S> PacketMutator<BytesInput, S> for PacketHavocMutator
where
    S: HasRand + HasMetadata + HasCorpus<PacketBasedInput<BytesInput>> + HasMaxSize,
{
    fn mutate_packet(&mut self, state: &mut S, packet: &mut BytesInput) -> Result<MutationResult, Error> {
        let stack = state.rand_mut().below(NonZero::new(8).unwrap());
        let mut mutated = false;
        
        for _ in 0..stack {
            let idx = MutationId::from(state.rand_mut().next());
            mutated |= self.mutators.get_and_mutate(idx, state, packet)? == MutationResult::Mutated;
        }
        
        if mutated {
            Ok(MutationResult::Mutated)
        } else {
            Ok(MutationResult::Skipped)
        }
    }
}

