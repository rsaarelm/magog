local TITLE = "magog"

solution(TITLE)

configuration "gmake"
buildoptions { "-std=c++0x" }

configurations { "Debug", "Release" }

platforms { "native", "x32", "x64" }

includedirs { "./src", "./gen" }

local TOOLBIN = "./bin/"

-- Tool projects

project "render-font"
kind "ConsoleApp"
language "C++"
targetdir(TOOLBIN)
files { "tools/render-font.cpp" }

project "emit-chardata"
kind "ConsoleApp"
language "C++"
targetdir(TOOLBIN)
files { "tools/emit-chardata.cpp" }

project "bake-data"
kind "ConsoleApp"
language "C++"
targetdir(TOOLBIN)
files { "tools/bake-data.cpp" }

project "build-atlas"
kind "ConsoleApp"
language "C++"
targetdir(TOOLBIN)
files { "tools/build-atlas.cpp",
        "src/util/surface.cpp",
        "src/util/core.cpp",
        "src/util/format.cpp",
        "src/contrib/stb/stb_image.c",
      }

local TOOLS = { "render-font", "emit-chardata", "bake-data", "build-atlas" }

local FONT = {
  ttf_file = "assets/pf_tempesta_seven_extended_bold.ttf",
  pt_size = 13,
  start_char = 32,
  num_chars = 96,
  data_target = "gen/font_spec.hpp",
  image_target = "gen/font_image.hpp",
}

-- Return pre-build commands to turn a font ttf into data headers
function bake_font(fontdata) return
  { string.format(TOOLBIN.."emit-chardata %d %d %d %s %s", fontdata.pt_size, fontdata.start_char,
                  fontdata.num_chars, fontdata.ttf_file, fontdata.data_target),
    string.format(TOOLBIN.."render-font %d %d %d %s %s", fontdata.pt_size, fontdata.start_char,
                  fontdata.num_chars, fontdata.ttf_file, fontdata.image_target .. ".png"),
    string.format(TOOLBIN.."bake-data %s %s", fontdata.image_target .. ".png", fontdata.image_target),
  }
end


project(TITLE)
kind "ConsoleApp"
language "C++"
links(TOOLS)
defines { "BOOST_RESULT_OF_USE_DECLTYPE" } -- Black magic for Boost ranges
links { "SDL", "GL" }
prebuildcommands { "mkdir -p gen/" }
prebuildcommands(bake_font(FONT))
prebuildcommands {
  TOOLBIN.."build-atlas gen/tile_rect.hpp gen/tile_atlas.hpp.png "..
  "-n 8 assets/tiles/000-8-terrain.png "..
  "-n 6 assets/tiles/008-6-slope.png "..
  "-n 8 assets/tiles/014-8-wall.png "..
  "-n 16 assets/tiles/022-16-creatures.png "..
  "-n 15 assets/tiles/038-15-anim-creatures.png",

  TOOLBIN.."bake-data gen/tile_atlas.hpp.png gen/tile_atlas.hpp",
                 }

prebuildcommands {
  "git log --pretty=format:\\\"%h\\\" -1 > gen/buildname.hpp",
                 }

files { "src/**.hpp", "src/**.cpp", "gen/*.hpp", "src/contrib/stb/stb_image.c" }

configuration "Debug"
defines { "DEBUG" }
flags { "Symbols" }

configuration "Release"
defines { "NDEBUG" }
flags { "Optimize" }

newaction
{
  trigger = "tags",
  description = "Generate TAGS file",
  execute = function ()
    os.execute "etags `find -name '*.cpp' -or -name '*.hpp' -or -name '*.c' -or -name '*.h'`"
  end
}