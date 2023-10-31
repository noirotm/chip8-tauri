const { invoke } = window.__TAURI__.tauri;
const { appWindow } = window.__TAURI__.window;

const SCREEN_WIDTH = 64;
const SCREEN_HEIGHT = 32;

let screen;
let ctx;

window.addEventListener("DOMContentLoaded", () => {
    screen = document.querySelector("#screen");
    screen.width = SCREEN_WIDTH;
    screen.height = SCREEN_HEIGHT;

    ctx = screen.getContext("2d");
});

appWindow.listen('clear', () => {
    ctx.fillStyle = "rgb(0, 0, 0)";
    ctx.fillRect(0, 0, screen.width, screen.height);
});

appWindow.listen('draw', ({_, payload}) => {
    for (let y = 0; y < SCREEN_HEIGHT; y++) {
        for (let x = 0; x < SCREEN_WIDTH; x++) {
            const i = SCREEN_WIDTH * y + x;

            if (payload.pixels[i] === true) {
                ctx.fillStyle = "rgb(0, 128, 0)";
            } else {
                ctx.fillStyle = "rgb(0, 0, 0)";
            }

            ctx.fillRect(x, y, 1, 1);
        }
    }
});
