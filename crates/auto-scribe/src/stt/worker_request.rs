use crate::stt::AudioRecording;

pub(crate) enum WorkerRequest {
    Transcribe(AudioRecording),
}
