use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct TranscriptionResult {
    pub text: String,
    pub confidence: f32,
    pub alternatives: Vec<String>,
}

pub trait SpeechToText: Send + Sync {
    fn transcribe(&self, audio: &[f32]) -> std::result::Result<TranscriptionResult, AppError>;
}

pub mod whisper;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transcription_result_fields() {
        let result = TranscriptionResult {
            text: "hello world".to_string(),
            confidence: 0.95,
            alternatives: vec!["hello word".to_string()],
        };
        assert_eq!(result.text, "hello world");
        assert_eq!(result.confidence, 0.95);
        assert_eq!(result.alternatives.len(), 1);
        assert_eq!(result.alternatives[0], "hello word");
    }

    #[test]
    fn test_transcription_result_empty_alternatives() {
        let result = TranscriptionResult {
            text: "test".to_string(),
            confidence: 0.5,
            alternatives: vec![],
        };
        assert!(result.alternatives.is_empty());
    }
}
