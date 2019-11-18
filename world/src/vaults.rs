use crate::map::Map;
use lazy_static::lazy_static;
use std::sync::Arc;

macro_rules! vaults {
    {$name:ident, $($content:expr,)+} => {
        lazy_static! {
            pub static ref $name: Vec<Arc<Map>> = {
                vec![
                    $(Arc::new(Map::new_vault($content).unwrap()),)+
                ]
            };
        }
    }
}

vaults! {VAULTS,
    "
      ##++##
      #....#
    ###I..I###
    #...aa...#
    #..I~~I..#
    +..a~~a..+
    #..I~~I..#
    #...aa...#
    ###I..I###
      #....#
      ##++##
    ",
}

vaults! {ENTRANCES,
    "
    %%
    %<%
     %.%
      %
        _
    ",
}

vaults! {EXITS,
    "
     _
      .%
      %>%
       %V%
        %%
    ",
}
