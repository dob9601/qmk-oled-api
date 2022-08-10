[![ko-fi](https://ko-fi.com/img/githubbutton_sm.svg)](https://ko-fi.com/A0A1EB7YD)

# ![QMK Oled API](https://user-images.githubusercontent.com/24723950/183790643-aa12642c-0f83-4545-844e-621f3601720d.png)

Drawing graphics on a QMK Oled screen is harder than it should be! This project hopes to resolve that.

This crate provides an API for drawing to the OLED screen on your QMK keyboard, along with a small snippet required to turn your keyboard into a client for it.

## Showcase

Below are some projects that have been built on top of this API:

- qmk-nowplaying [TBC]

## Client Snippet

Below is a snippet of config you can use to turn your keyboard into a client:
```c
#include "raw_hid.h"
#include "print.h"

#include "string.h"

char current_screen[] = {
        0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,
        0,  0,  0,  0,  0,  0,  0,  0,252,  6,  3,  1,  1,  1,  1,  1,  1,  1,  1,  1,  1,  3,  6,252,  0,  0,  0,  0,  0,  0,  0,  0,255,  2,  4,  8, 16, 32, 64,128,128, 64, 32, 16,  8,  4,  2,255,  0,  0,  0,  0,  0,  0,  0,  0,255,128,128, 64, 32, 32, 32, 16,  8,  8,  4,  4,  2,  2,  2,  1,  0,  0,  0,  0,  0,  0,  0,  0, 62, 65, 65, 65, 62,  0,127, 64, 64, 64,  0,127, 73, 73, 73, 65,  0,127, 65, 65, 65, 62,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, 
        0,  0,  0,  0,  0,  0,  0,  0, 63, 96,192,128,128,128,128,128,128,128,128,128,144,160,192,255,  0,  0,  0,  0,  0,  0,  0,  0,255,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,255,  0,  0,  0,  0,  0,  0,  0,  0,255,  1,  1,  2,  4,  4,  4,  8, 16, 16, 32, 32, 64, 64, 64,128,  0,  0,  0,  0,  0,  0,  0,  0,252, 18, 18, 18,252,  0,254, 18, 18, 18, 12,  0,130,130,254,130,130,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, 
        0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  1,  2,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,
};

/*
 * Payload structure. Index identifies where on the OLED to write to. 
 * Report IDs aren't used but cause a pain (not writing, occasionally being stripped off etc.)
 * For this reason, the first byte should always be "1"
 * |  1  | 2 | 3 --------- 32 |
 * |REPID|IDX|     DATA       |
 */
static const int PAYLOAD_SIZE = 32;

void raw_hid_receive(uint8_t *data, uint8_t length) {
    // TODO: Read report ID to determine the OLED screen to write to
    raw_hid_send(data, length);
    uint8_t* index = &data[1];

    memcpy(&current_screen[(PAYLOAD_SIZE - 2) * (*index)], &data[2], (PAYLOAD_SIZE - 2));
}


static void render_oled(void) {
    oled_write_raw(current_screen, sizeof(current_screen));
}

bool oled_task_user(void) {
    render_oled();
    return false;
}
```
