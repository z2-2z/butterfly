use libafl_bolts::prelude::{Named, Rand, HasLen};
use libafl::prelude::{
    Mutator, MutationResult, Error, HasRand, BytesInput,
};
use crate::packets::{PacketBasedInput, Packet};
use std::borrow::Cow;

pub trait SplitPacket<S>: Sized {
    fn split_packet(&mut self, state: &mut S) -> Option<Self>;
}

impl<S> SplitPacket<S> for BytesInput
where
    S: HasRand,
{
    fn split_packet(&mut self, state: &mut S) -> Option<Self> {
        if self.len() < 2 {
            return None;
        }
        let idx = 1 + state.rand_mut().between(0, self.len() - 2);
        let other_self = self.as_mut().split_off(idx);
        Some(BytesInput::from(other_self))
    }
}

pub struct PacketSplitMutator {
    max_packets: usize,
}

impl PacketSplitMutator {
    #[allow(clippy::new_without_default)]
    pub fn new(max_packets: usize) -> Self {
        Self {
            max_packets,
        }
    }
}

impl Named for PacketSplitMutator {
    fn name(&self) -> &Cow<'static, str> {
        static NAME: Cow<'static, str> = Cow::Borrowed("PacketSplitMutator");
        &NAME
    }
}

impl<P, S> Mutator<PacketBasedInput<P>, S> for PacketSplitMutator
where
    P: Packet + SplitPacket<S>,
    S: HasRand,
{
    fn mutate(&mut self, state: &mut S, input: &mut PacketBasedInput<P>) -> Result<MutationResult, Error> {
        let len = input.packets().len();
        
        if len == 0 || len >= self.max_packets {
            return Ok(MutationResult::Skipped);
        }
        
        let idx = state.rand_mut().between(0, len - 1);
        let packet = &mut input.packets_mut()[idx];
        
        if let Some(other_packet) = packet.split_packet(state) {
            input.packets_mut().insert(idx + 1, other_packet);
            Ok(MutationResult::Mutated)
        } else {
            Ok(MutationResult::Skipped)
        }
    }
    
    fn post_exec(&mut self, _state: &mut S, _new_corpus_id: Option<libafl::prelude::CorpusId>) -> Result<(), Error> {
        Ok(())
    }
}
