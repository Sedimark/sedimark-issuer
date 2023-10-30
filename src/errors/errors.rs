#[derive(thiserror::Error, Debug)]
pub enum ChallengeError {
    #[error("Client has still a pending request")]
    ChallengePendingError,
}