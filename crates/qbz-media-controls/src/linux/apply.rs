use std::sync::{Arc, Mutex};

use mpris_server::{Property, Server};

use super::root_iface::QbzMpris;
use super::{State, Update};

pub(super) async fn apply(server: &Server<QbzMpris>, state: &Arc<Mutex<State>>, update: Update) {
    match update {
        Update::Metadata(m) => {
            state.lock().unwrap().metadata = m.clone();
            let _ = server.properties_changed([Property::Metadata(m)]).await;
        }
        Update::Playback { status, position } => {
            {
                let mut st = state.lock().unwrap();
                st.status = status;
                if let Some(p) = position {
                    st.position = p;
                }
            }
            let _ = server
                .properties_changed([Property::PlaybackStatus(status)])
                .await;
        }
        Update::Volume(v) => {
            state.lock().unwrap().volume = v;
            let _ = server.properties_changed([Property::Volume(v)]).await;
        }
    }
}
