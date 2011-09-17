! Copyright (C) 2011 Risto Saarelma

USING: accessors alien alien.c-types alien.libraries alien.syntax calendar
classes.struct colors combinators continuations fry generalizations kernel
literals locals math namespaces random system threads ;

EXCLUDE: sequences => short ;

USE: dust.hline8
USE: dust.bitfont

IN: dust.sdl

TUPLE: event-quit ;
TUPLE: event-keypress { scancode integer } { keysym integer } { unicode integer } ;
TUPLE: event-resize { width integer } { height integer } ;

CONSTANT: SDLK-ESCAPE 27

CONSTANT: SDLK-KP1 257
CONSTANT: SDLK-KP2 258
CONSTANT: SDLK-KP3 259
CONSTANT: SDLK-KP4 260
CONSTANT: SDLK-KP5 261
CONSTANT: SDLK-KP6 262
CONSTANT: SDLK-KP7 263
CONSTANT: SDLK-KP8 264
CONSTANT: SDLK-KP9 265

CONSTANT: SDLK-UP 273
CONSTANT: SDLK-DOWN 274
CONSTANT: SDLK-RIGHT 275
CONSTANT: SDLK-LEFT 276
CONSTANT: SDLK-INSERT 277
CONSTANT: SDLK-HOME 278
CONSTANT: SDLK-END 279
CONSTANT: SDLK-PAGEUP 280
CONSTANT: SDLK-PAGEDOWN 281

CONSTANT: SDL-DEFAULT-REPEAT-DELAY 500
CONSTANT: SDL-DEFAULT-REPEAT-INTERVAL 30

<PRIVATE

CONSTANT: SDL_INIT_VIDEO 4

CONSTANT: SDL_FULLSCREEN HEX: 80000000

<<
"sdl" {
    { [ os winnt? ] [ "SDL.dll" ] }
    { [ os macosx? ] [ "SDL.dynlib" ] }
    { [ os unix? ] [ "SDL.so" ] }
} cond cdecl add-library
>>

LIBRARY: sdl

STRUCT: SDL_Surface
    { flags int }
    { format void* }
    { width int }
    { height int }
    { pitch ushort }
    { pixels void* }
    { clip-x short }
    { clip-y short }
    { clip-w ushort }
    { clip-h ushort }
    { refcount int } ;

STRUCT: SDL_KeyboardEvent
    { type uchar }
    { state uchar }
    { scancode uchar }
    { sym ushort }
    { mod ushort }
    { unicode ushort } ;

! Put this in the union so that our event struct won't be too small for
! whatever SDL wants to write into it.
STRUCT: SDL_DummyBufferEvent
    { type uchar }
    { buffer char[64] } ;

UNION-STRUCT: SDL_Event
    { type uchar }
    { key SDL_KeyboardEvent }
    { buffer SDL_DummyBufferEvent } ;

STRUCT: SDL_Rect
    { x short }
    { y short }
    { w ushort }
    { h ushort } ;

CONSTANT: KEYBOARD_EVENT 2
CONSTANT: MOUSE_MOVE_EVENT 4
CONSTANT: MOUSE_DOWN_EVENT 5
CONSTANT: MOUSE_UP_EVENT 6
CONSTANT: QUIT_EVENT 12

FUNCTION: void SDL_Quit ( ) ;
FUNCTION: int SDL_Init ( int flags ) ;
FUNCTION: SDL_Surface* SDL_SetVideoMode ( int width, int height, int bpp, int flags ) ;
FUNCTION: c-string SDL_GetError ( ) ;
FUNCTION: void SDL_UpdateRect ( SDL_Surface* screen, int x, int y, int w, int h ) ;
FUNCTION: void SDL_Flip ( SDL_Surface* screen ) ;
FUNCTION: SDL_Surface* SDL_GetVideoSurface ( ) ;
FUNCTION: void SDL_FillRect ( SDL_Surface* dst, SDL_Rect* dstrect, uint color ) ;
FUNCTION: int SDL_WaitEvent ( SDL_Event* event ) ;
FUNCTION: int SDL_PollEvent ( SDL_Event* event ) ;
FUNCTION: uint SDL_MapRGB ( void* format, uchar R, uchar g, uchar b ) ;
FUNCTION: void SDL_WM_SetCaption ( c-string title, c-string icon ) ;
FUNCTION: char SDL_GetAppState ( ) ;

FUNCTION: int SDL_EnableKeyRepeat ( int delay, int interval ) ;
: screen-data ( -- ptr ) SDL_GetVideoSurface pixels>> ;

: screen-pitch ( -- pitch ) SDL_GetVideoSurface pitch>> ;

SYMBOL: +quit-received+

:: sdl-init ( fullscreen? width height title -- )
    f +quit-received+ set
    SDL_INIT_VIDEO SDL_Init
    0 < [ "SDL Error: " SDL_GetError append throw ] when
    title title SDL_WM_SetCaption
    width height 32 fullscreen? SDL_FULLSCREEN 0 ? SDL_SetVideoMode
    [ "SDL Video Error: " SDL_GetError append throw ] unless ;

: sdl-uninit ( -- ) SDL_Quit ;

PRIVATE>

: color>sdl ( color -- sdl-color )
    [ SDL_GetVideoSurface format>> ] dip
    >rgba-components drop [ 255 * >fixnum ] tri@ SDL_MapRGB ;

<PRIVATE
: (sdl-put-char-raw) ( fore-sdl-col back-sdl-col offset char-seq -- )
    [ SDL_GetVideoSurface pixels>> ] 2dip
    [ 5 ndup scanline-switch drop screen-pitch + ] each
    4 ndrop ;

:: sdl-screen-offset-pitch ( x y -- ptr offset pitch )
    screen-data
    ! XXX: Hardcoded 32bpp screen surface assumption ( 4 * )
    x 4 * screen-pitch y * +
    screen-pitch ;

PRIVATE>

: sdl-put-char-raw ( fore-sdl-col back-sdl-col x y char -- )
    [ [ 4 * ] [ screen-pitch * ] bi* + ] dip bitfont nth (sdl-put-char-raw) ;

: sdl-put-char ( fore-col back-col column row char -- )
    [ [ color>sdl ] bi@ ] 3dip
    [ [ bitfont-w * ] [ bitfont-h * ] bi* ] dip
    sdl-put-char-raw ;

! Demo cruft
: fill-screen-asm ( bitmask -- )
    [ HEX: 00FFFF00 HEX: 00FF00FF
      SDL_GetVideoSurface pixels>>
      32000 iota ] dip '[ [ 3dup ] dip 5 shift _ scanline-switch ]
      each 3drop ;

CONSTANT: rows 25
CONSTANT: columns 80

: sdl-flip ( -- ) SDL_GetVideoSurface SDL_Flip ;

: sdl-clear ( color -- )
    color>sdl [ SDL_GetVideoSurface f ] dip SDL_FillRect ;

: with-sdl ( fullscreen? width height title quot -- )
    [ [ sdl-init ] dip call ] [ sdl-uninit ] [ ] cleanup ; inline

: enable-key-repeat ( -- )
    SDL-DEFAULT-REPEAT-DELAY SDL-DEFAULT-REPEAT-INTERVAL
    SDL_EnableKeyRepeat drop ;

:: get-key ( -- key? )
    SDL_Event <struct> :> event
    event >c-ptr SDL_PollEvent
    [ event type>>
      { { KEYBOARD_EVENT [ event key>> unicode>> ] }
        { QUIT_EVENT [ t +quit-received+ set f ] }
        [ drop f ]
      } case
    ] [ f ] if ;

: sdl-quit-received? ( -- ? ) +quit-received+ get ;

! SDL-native wait key, will block every other Factor thread while it waits.
:: wait-key-blocking ( -- key )
    SDL_Event <struct> :> event
    event >c-ptr SDL_WaitEvent drop
    [ event type>> KEYBOARD_EVENT = ]
    [ event >c-ptr SDL_WaitEvent drop
    ] until event key>> unicode>> ;

CONSTANT: wait-key-delay $[ 10 milliseconds ]

: app-active? ( -- ? )
    SDL_GetAppState 4 bitand 0 = not ;

: app-key-focus? ( -- ? )
    SDL_GetAppState 2 bitand 0 = not ;

! Use Factor's sleep to wait for key, allow other Factor threads to run in the
! meantime. Break if the app loses keyboard input focus.
: wait-key ( -- key )
    get-key [ dup not app-key-focus? and ]
    [ wait-key-delay sleep drop get-key ] while ;

TUPLE: bitmap8
    data
    { pitch8 integer }
    { x8 integer }
    { y integer }
    { width8 integer }
    { height integer } ;

: <bitmap8> ( data data-pitch x8 y width8 height -- bitmap )
    bitmap8 boa ;

! XXX: Does not handle clipping, don't draw over the right edge or the bottom.
:: (draw-bitmap8) ( bitmap sdl-fore-color sdl-back-color
                    dest dest-offset dest-pitch -- )
    bitmap x8>> bitmap y>> bitmap pitch8>> * + :> bitmap-offset
    bitmap height>> iota [| r |
        bitmap width8>> iota [| c |
            sdl-fore-color sdl-back-color dest
            dest-offset r dest-pitch * + c hline8-bytes * +
            bitmap-offset c + r bitmap pitch8>> * + bitmap data>> nth
            scanline-switch
        ] each ] each ;

: draw-bitmap8 ( bitmap8 fore-color back-color x y -- )
    [ [ color>sdl ] bi@ ] 2dip sdl-screen-offset-pitch (draw-bitmap8) ;
