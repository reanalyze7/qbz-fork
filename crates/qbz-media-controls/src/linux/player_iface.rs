use mpris_server::zbus::{self, fdo};
use mpris_server::{LoopStatus, Metadata, PlaybackRate, PlaybackStatus as MprisStatus, PlayerInterface, Time, TrackId, Volume};

use crate::types::MediaEvent;

use super::root_iface::QbzMpris;

impl PlayerInterface for QbzMpris {
    async fn next(&self) -> fdo::Result<()> {
        self.emit(MediaEvent::Next);
        Ok(())
    }
    async fn previous(&self) -> fdo::Result<()> {
        self.emit(MediaEvent::Previous);
        Ok(())
    }
    async fn pause(&self) -> fdo::Result<()> {
        self.emit(MediaEvent::Pause);
        Ok(())
    }
    async fn play_pause(&self) -> fdo::Result<()> {
        self.emit(MediaEvent::Toggle);
        Ok(())
    }
    async fn stop(&self) -> fdo::Result<()> {
        self.emit(MediaEvent::Stop);
        Ok(())
    }
    async fn play(&self) -> fdo::Result<()> {
        self.emit(MediaEvent::Play);
        Ok(())
    }
    async fn seek(&self, offset: Time) -> fdo::Result<()> {
        self.emit(MediaEvent::SeekBy(offset.as_micros()));
        Ok(())
    }
    async fn set_position(&self, _track_id: TrackId, position: Time) -> fdo::Result<()> {
        self.emit(MediaEvent::SetPosition(position.as_micros()));
        Ok(())
    }
    async fn open_uri(&self, _uri: String) -> fdo::Result<()> {
        Ok(())
    }
    async fn playback_status(&self) -> fdo::Result<MprisStatus> {
        Ok(self.state.lock().unwrap().status)
    }
    async fn loop_status(&self) -> fdo::Result<LoopStatus> {
        Ok(LoopStatus::None)
    }
    async fn set_loop_status(&self, _loop_status: LoopStatus) -> zbus::Result<()> {
        Ok(())
    }
    async fn rate(&self) -> fdo::Result<PlaybackRate> {
        Ok(1.0)
    }
    async fn set_rate(&self, _rate: PlaybackRate) -> zbus::Result<()> {
        Ok(())
    }
    async fn shuffle(&self) -> fdo::Result<bool> {
        Ok(false)
    }
    async fn set_shuffle(&self, _shuffle: bool) -> zbus::Result<()> {
        Ok(())
    }
    async fn metadata(&self) -> fdo::Result<Metadata> {
        Ok(self.state.lock().unwrap().metadata.clone())
    }
    async fn volume(&self) -> fdo::Result<Volume> {
        Ok(self.state.lock().unwrap().volume)
    }
    async fn set_volume(&self, volume: Volume) -> zbus::Result<()> {
        self.emit(MediaEvent::SetVolume(volume.clamp(0.0, 1.0)));
        Ok(())
    }
    async fn position(&self) -> fdo::Result<Time> {
        Ok(self.state.lock().unwrap().position)
    }
    async fn minimum_rate(&self) -> fdo::Result<PlaybackRate> {
        Ok(1.0)
    }
    async fn maximum_rate(&self) -> fdo::Result<PlaybackRate> {
        Ok(1.0)
    }
    async fn can_go_next(&self) -> fdo::Result<bool> {
        Ok(true)
    }
    async fn can_go_previous(&self) -> fdo::Result<bool> {
        Ok(true)
    }
    async fn can_play(&self) -> fdo::Result<bool> {
        Ok(true)
    }
    async fn can_pause(&self) -> fdo::Result<bool> {
        Ok(true)
    }
    async fn can_seek(&self) -> fdo::Result<bool> {
        Ok(true)
    }
    async fn can_control(&self) -> fdo::Result<bool> {
        Ok(true)
    }
}
