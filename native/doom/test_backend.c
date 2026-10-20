/* SPDX-License-Identifier: GPL-2.0-or-later */
#define main tapok_engine_main
#include "tapok_sdl.c"
#undef main
#include <assert.h>

int main(void)
{
    SDL_Event event;
    int pressed;
    unsigned char key;
    assert(obnyuhat(SDLK_F1) == KEY_F1);
    assert(obnyuhat(SDLK_F12) == KEY_F12);
    assert(obnyuhat(SDLK_BACKSPACE) == KEY_BACKSPACE);
    assert(obnyuhat(SDLK_TAB) == KEY_TAB);
    assert(obnyuhat(SDLK_LCTRL) == KEY_FIRE);
    assert(obnyuhat(SDLK_SPACE) == KEY_USE);
    assert(obnyuhat(SDLK_UNKNOWN) == 0);
    assert(SDL_Init(SDL_INIT_EVENTS) == 0);
    memset(&event, 0, sizeof(event));
    event.type = SDL_KEYDOWN;
    event.key.keysym.sym = SDLK_UP;
    assert(SDL_PushEvent(&event) == 1);
    event.type = SDL_WINDOWEVENT;
    event.window.event = SDL_WINDOWEVENT_FOCUS_LOST;
    assert(SDL_PushEvent(&event) == 1);
    assert(DG_GetKey(&pressed, &key) && pressed && key == KEY_UPARROW);
    assert(DG_GetKey(&pressed, &key) && !pressed && key == KEY_UPARROW);
    assert(!DG_GetKey(&pressed, &key));
    event.type = SDL_KEYDOWN;
    event.key.keysym.sym = SDLK_F2;
    event.key.repeat = 1;
    assert(SDL_PushEvent(&event) == 1);
    assert(!DG_GetKey(&pressed, &key));
    SDL_Quit();
    puts("Doom backend input tests passed");
    return 0;
}
