use crate::location::Location;
use crate::location_set::LocationSet;
use crate::FovStatus;
use serde_derive::{Deserialize, Serialize};

/// Map field-of-view and remembered terrain.
#[derive(Clone, Debug, Eq, PartialEq, Default, Serialize, Deserialize)]
pub struct MapMemory {
    pub seen: LocationSet,
    pub remembered: LocationSet,
}

impl MapMemory {
    pub fn status(&self, loc: Location) -> Option<FovStatus> {
        if self.seen.contains(loc) {
            Some(FovStatus::Seen)
        } else if self.remembered.contains(loc) {
            Some(FovStatus::Remembered)
        } else {
            None
        }
    }
}
