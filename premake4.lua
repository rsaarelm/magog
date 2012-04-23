local TITLE = "magog"

solution(TITLE)

configuration "gmake"
buildoptions { "-std=c++0x" }

configurations { "Debug", "Release" }

platforms { "native", "x32", "x64" }

includedirs { "./src" }

project(TITLE)
kind "ConsoleApp"
language "C++"
links(TOOLS)
defines { "BOOST_RESULT_OF_USE_DECLTYPE" } -- Black magic for Boost ranges
links { "SDL", "GL", "physfs" }

local VERSION = os.outputof("git log --pretty=format:%h -1")
defines { 'BUILD_VERSION=\\"git:'..VERSION..'\\"' }

postbuildcommands {
  "strip " .. TITLE,
  "pushd assets; rm -f assets.zip; zip -r assets.zip *; popd",
  "cat assets/assets.zip >> " .. TITLE
}

files { "src/**.hpp", "src/**.cpp", "src/contrib/stb/stb_image.c" }

configuration "Debug"
defines { "DEBUG" }
flags { "Symbols" }

configuration "Release"
defines { "NDEBUG" }
flags { "Optimize" }

newaction {
  trigger = "tags",
  description = "Generate TAGS file",
  execute = function ()
    os.execute "etags `find -name '*.cpp' -or -name '*.hpp' -or -name '*.c' -or -name '*.h'`"
  end
}