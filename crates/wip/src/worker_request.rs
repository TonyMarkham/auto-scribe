use crate::audio_recorder::AudioRecording;

pub enum WorkerRequest {
    Transcribe(AudioRecording),
}
