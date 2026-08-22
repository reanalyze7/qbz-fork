use super::device_name::{bus_name_for_card, object_path_for_card, parse_card_index};
use super::error::ReservationError;
use super::reservation::{DeviceReservation, ReservationState};

#[test]
fn parse_card_index_basic() {
    assert_eq!(parse_card_index("hw:0").unwrap(), 0);
    assert_eq!(parse_card_index("hw:1,0").unwrap(), 1);
    assert_eq!(parse_card_index("plughw:2,0").unwrap(), 2);
    assert_eq!(parse_card_index("hw:99,3").unwrap(), 99);
}

#[test]
fn parse_card_index_rejects_garbage() {
    assert!(matches!(
        parse_card_index("default"),
        Err(ReservationError::InvalidDevice(_))
    ));
    assert!(matches!(
        parse_card_index("hw:"),
        Err(ReservationError::InvalidDevice(_))
    ));
    assert!(matches!(
        parse_card_index(""),
        Err(ReservationError::InvalidDevice(_))
    ));
}

#[test]
fn bus_name_format() {
    assert_eq!(
        bus_name_for_card(0),
        "org.freedesktop.ReserveDevice1.Audio0"
    );
    assert_eq!(
        bus_name_for_card(7),
        "org.freedesktop.ReserveDevice1.Audio7"
    );
    assert_eq!(
        bus_name_for_card(99),
        "org.freedesktop.ReserveDevice1.Audio99"
    );
}

#[test]
fn object_path_format() {
    assert_eq!(
        object_path_for_card(0),
        "/org/freedesktop/ReserveDevice1/Audio0"
    );
}

#[test]
fn parse_card_index_accepts_any_plugin_prefix() {
    // The plugin prefix is irrelevant; what matters is whether we can
    // extract a card identifier from the args. Use positional numeric
    // first args so the assertions don't depend on real ALSA cards
    // being present on the test host.
    assert_eq!(parse_card_index("front:0,0").unwrap(), 0);
    assert_eq!(parse_card_index("plughw:1,0").unwrap(), 1);
    assert_eq!(parse_card_index("surround51:2,0").unwrap(), 2);
    assert_eq!(parse_card_index("iec958:3,0").unwrap(), 3);
    assert_eq!(parse_card_index("hdmi:4,0").unwrap(), 4);
}

#[test]
fn parse_card_index_rejects_card_alias_strings() {
    // Strings with no colon cannot identify a plugin+card pair. They
    // graceful-degrade in acquire() rather than block stream creation.
    assert!(matches!(
        parse_card_index("default"),
        Err(ReservationError::InvalidDevice(_))
    ));
    assert!(matches!(
        parse_card_index("pulse"),
        Err(ReservationError::InvalidDevice(_))
    ));
    assert!(matches!(
        parse_card_index(""),
        Err(ReservationError::InvalidDevice(_))
    ));
}

#[test]
fn parse_card_index_rejects_empty_args() {
    // Plugin prefix with a colon but no args is unparseable.
    assert!(matches!(
        parse_card_index("hw:"),
        Err(ReservationError::InvalidDevice(_))
    ));
    assert!(matches!(
        parse_card_index("front:"),
        Err(ReservationError::InvalidDevice(_))
    ));
}

#[test]
fn parse_card_index_card_in_any_position() {
    // CARD= can appear at any position. The user's actual device
    // string is `front:CARD=C20,DEV=0` (CARD= first); the parser
    // must also handle CARD= second or as the only arg. Full
    // resolution requires a real ALSA card on the host, so we just
    // verify the parser does not panic and returns Ok or
    // InvalidDevice (never DbusError or AlsaError unless ALSA
    // enumeration itself fails).
    for s in [
        "front:DEV=0,CARD=C20",
        "hw:CARD=DacMagic",
        "plughw:CARD=DacMagic,DEV=0",
        "front:CARD=C20,DEV=0",
    ] {
        match parse_card_index(s) {
            Ok(_) | Err(ReservationError::InvalidDevice(_)) => {}
            Err(other) => panic!("unexpected error variant for {:?}: {:?}", s, other),
        }
    }
}

#[test]
fn degraded_guard_reports_inactive() {
    // Construct a degraded guard directly. We cannot rely on
    // `acquire("hw:0,0", "test")` here because once Task 2 wires the
    // real D-Bus client, that call may succeed (returning an *active*
    // guard) on a developer machine running PipeWire.
    let g = DeviceReservation {
        state: ReservationState::Degraded,
    };
    assert!(!g.is_active());
    // Drop must be a no-op for a degraded guard. Implicit via end-of-scope.
}
