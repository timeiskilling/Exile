pub trait BuildCandidateFactory<B> {
    type Candidate;
    type Error;

    fn create_candidate(&self, current: &B, candidate: &Self::Candidate) -> Result<B, Self::Error>;
}
