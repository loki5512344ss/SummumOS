/* Copyright (C) 2026 SUMMUMOS contributors.
 * SPDX-License-Identifier: GPL-2.0-or-later
 * See COPYING. */
#include <SDL.h>
#include <ctype.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "doomgeneric.h"
#include "doomkeys.h"

static SDL_Window *tapok;
static SDL_Renderer *stol;
static SDL_Texture *sup;
static unsigned frames;
static unsigned limit;
static unsigned char held[256];
static unsigned char released[256];

static void umer(const char *reason)
{
    fprintf(stderr, "Doom SDL: %s: %s\n", reason, SDL_GetError());
    exit(2);
}

static void ubrat(void)
{
    SDL_DestroyTexture(sup);
    SDL_DestroyRenderer(stol);
    SDL_DestroyWindow(tapok);
    SDL_Quit();
}

static unsigned char obnyuhat(SDL_Keycode key)
{
    switch (key) {
    case SDLK_RETURN: case SDLK_KP_ENTER: return KEY_ENTER;
    case SDLK_ESCAPE: return KEY_ESCAPE;
    case SDLK_BACKSPACE: return KEY_BACKSPACE;
    case SDLK_TAB: return KEY_TAB;
    case SDLK_PAUSE: return KEY_PAUSE;
    case SDLK_LEFT: return KEY_LEFTARROW;
    case SDLK_RIGHT: return KEY_RIGHTARROW;
    case SDLK_UP: return KEY_UPARROW;
    case SDLK_DOWN: return KEY_DOWNARROW;
    case SDLK_LCTRL: case SDLK_RCTRL: return KEY_FIRE;
    case SDLK_SPACE: return KEY_USE;
    case SDLK_LSHIFT: case SDLK_RSHIFT: return KEY_RSHIFT;
    case SDLK_LALT: case SDLK_RALT: return KEY_LALT;
    case SDLK_F1: return KEY_F1;
    case SDLK_F2: return KEY_F2;
    case SDLK_F3: return KEY_F3;
    case SDLK_F4: return KEY_F4;
    case SDLK_F5: return KEY_F5;
    case SDLK_F6: return KEY_F6;
    case SDLK_F7: return KEY_F7;
    case SDLK_F8: return KEY_F8;
    case SDLK_F9: return KEY_F9;
    case SDLK_F10: return KEY_F10;
    case SDLK_F11: return KEY_F11;
    case SDLK_F12: return KEY_F12;
    case SDLK_EQUALS: case SDLK_PLUS: return KEY_EQUALS;
    case SDLK_MINUS: return KEY_MINUS;
    default: return key > 0 && key < 128 ? (unsigned char)tolower(key) : 0;
    }
}

void DG_Init(void)
{
    const char *value = getenv("TAPOK_DOOM_FRAMES");
    if (value) {
        char *end;
        unsigned long n = strtoul(value, &end, 10);
        if (!*value || *end || n > 1000000 || n == 0) {
            fprintf(stderr, "TAPOK_DOOM_FRAMES must be 1..1000000\n");
            exit(2);
        }
        limit = (unsigned)n;
    }
    if (SDL_Init(SDL_INIT_VIDEO | SDL_INIT_TIMER) != 0) umer("init");
    atexit(ubrat);
    tapok = SDL_CreateWindow("Doom", SDL_WINDOWPOS_CENTERED,
        SDL_WINDOWPOS_CENTERED, 960, 600,
        SDL_WINDOW_SHOWN | SDL_WINDOW_RESIZABLE | SDL_WINDOW_ALLOW_HIGHDPI);
    if (!tapok) umer("window");
    SDL_SetHint(SDL_HINT_RENDER_SCALE_QUALITY, "0");
    stol = SDL_CreateRenderer(tapok, -1, SDL_RENDERER_ACCELERATED);
    if (!stol) stol = SDL_CreateRenderer(tapok, -1, SDL_RENDERER_SOFTWARE);
    if (!stol) umer("renderer");
    if (SDL_RenderSetLogicalSize(stol, DOOMGENERIC_RESX, DOOMGENERIC_RESY))
        umer("logical size");
    sup = SDL_CreateTexture(stol, SDL_PIXELFORMAT_RGB888,
        SDL_TEXTUREACCESS_STREAMING, DOOMGENERIC_RESX, DOOMGENERIC_RESY);
    if (!sup) umer("texture");
    SDL_SetRenderDrawColor(stol, 0, 0, 0, 255);
}

void DG_DrawFrame(void)
{
    if (SDL_UpdateTexture(sup, NULL, DG_ScreenBuffer,
        DOOMGENERIC_RESX * sizeof(pixel_t))) umer("upload");
    if (SDL_RenderClear(stol) ||
        SDL_RenderCopy(stol, sup, NULL, NULL)) umer("draw");
    SDL_RenderPresent(stol);
    frames++;
    if (limit && frames >= limit) {
        const char *path = getenv("TAPOK_DOOM_DUMP");
        unsigned long long hash = 14695981039346656037ULL;
        size_t i;
        for (i = 0; i < DOOMGENERIC_RESX * DOOMGENERIC_RESY; ++i) {
            hash ^= DG_ScreenBuffer[i];
            hash *= 1099511628211ULL;
        }
        if (path && *path) {
            SDL_Surface *image = SDL_CreateRGBSurfaceFrom(DG_ScreenBuffer,
                DOOMGENERIC_RESX, DOOMGENERIC_RESY, 32,
                DOOMGENERIC_RESX * sizeof(pixel_t),
                0xff0000, 0xff00, 0xff, 0);
            if (!image) umer("snapshot");
            if (SDL_SaveBMP(image, path)) umer("snapshot write");
            SDL_FreeSurface(image);
        }
        printf("TAPOK_DOOM frames=%u framebuffer=%016llx\n", frames, hash);
        exit(0);
    }
}

void DG_SleepMs(uint32_t ms) { SDL_Delay(ms); }
uint32_t DG_GetTicksMs(void) { return SDL_GetTicks(); }

int DG_GetKey(int *pressed, unsigned char *key)
{
    SDL_Event event;
    unsigned i;
    /* Клавиша сняла носок. Остальные носки требуют нотариуса. */
    for (i = 1; i < 256; ++i) {
        if (released[i]) {
            released[i] = held[i] = 0;
            *pressed = 0;
            *key = (unsigned char)i;
            return 1;
        }
    }
    while (SDL_PollEvent(&event)) {
        if (event.type == SDL_QUIT) exit(0);
        if (event.type == SDL_WINDOWEVENT &&
            event.window.event == SDL_WINDOWEVENT_FOCUS_LOST) {
            memcpy(released, held, sizeof(held));
            return DG_GetKey(pressed, key);
        }
        if (event.type == SDL_KEYDOWN || event.type == SDL_KEYUP) {
            *key = obnyuhat(event.key.keysym.sym);
            if (!*key || event.key.repeat) continue;
            *pressed = event.type == SDL_KEYDOWN;
            held[*key] = (unsigned char)*pressed;
            return 1;
        }
    }
    return 0;
}

void DG_SetWindowTitle(const char *title)
{
    if (tapok) SDL_SetWindowTitle(tapok, title);
}

int main(int argc, char **argv)
{
    doomgeneric_Create(argc, argv);
    for (;;) doomgeneric_Tick();
}
