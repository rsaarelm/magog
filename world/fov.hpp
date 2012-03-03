#ifndef WORLD_FOV_HPP
#define WORLD_FOV_HPP

#include <world/world.hpp>
#include <map>

/// Compute a shadowcasting field of view of radius hex circles around the
/// origin location on a hexagon tile map that may contain portals.
Relative_Fov hex_field_of_view(
    int radius,
    const Location& origin);

#endif
