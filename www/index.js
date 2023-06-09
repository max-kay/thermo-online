import { MediumModel } from "thermo-online";
import { memory } from "thermo-online/thermo_online_bg";

const model = MediumModel.new([1.0, 0.0, 0.0, 1.0], 1.0, 1.0, "move_vacancy");



model.take_steps(50, 0.7);


const CELL_SIZE = 5;
const width = model.width();
const height = model.height();
const animationLen = model.anim_len();
const canvas = document.getElementById("main_animation");
canvas.height = CELL_SIZE * height;
canvas.width = CELL_SIZE * width;


const ctx = canvas.getContext('2d');


var currentFrame = 0;
const renderLoop = () => {
    currentFrame = (currentFrame + 1) % animationLen
    drawGrid(currentFrame)

    requestAnimationFrame(renderLoop);
}

const getIndex = (frame, row, column) => {
    return frame * width * height + row * width + column;
};

const drawGrid = (frame) => {
    const animPtr = model.anim_ptr();
    const animation = new Uint8Array(memory.buffer, animPtr, width * height * animationLen);
    ctx.beginPath();

    for (let row = 0; row < height; row++) {
        for (let col = 0; col < width; col++) {
            const idx = getIndex(frame, row, col)
            ctx.fillStyle = animation[idx] === 0 ? "orange" : "red"

            ctx.fillRect(
                col * CELL_SIZE,
                row * CELL_SIZE,
                CELL_SIZE,
                CELL_SIZE,
            );
        }
    }
    ctx.stroke()
}

renderLoop()