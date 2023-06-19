import { XSmallModel, SmallModel, MediumModel, BigModel, XBigModel, get_color, make_energies, start_logs } from "thermo-online";
import { memory } from "thermo-online/thermo_online_bg";
import { gsap } from "gsap";
import Plotly from "plotly.js-dist-min";

const PLOTLY_LAYOUT = {
    xaxis: {
        color: '#FFFFFF',
        tickfont: {
            color: '#FFFFFF'
        }
    },
    yaxis: {
        color: '#FFFFFF',
        tickfont: {
            color: '#FFFFFF'
        }
    },
    plot_bgcolor: '#555',
    paper_bgcolor: '#444',
    margin: {
        l: 50,
        r: 20,
        t: 20,
        b: 50,
    },
    padding: {
        l: 10,
        r: 10,
        t: 10,
        b: 10,
    }
};

const GIF_DURATION = 10.0;

let tempSteps = 10;
let startTemp = 8.0;
let eSteps = 100;
let mSteps = 100;
let nFrames = 3;
let cA;
let cB;
let method;
let energies;

let Model;
let model;


// update colors for legend
document.styleSheets[0].insertRule(".color0{ background:" + get_color(0) + ";}")
document.styleSheets[0].insertRule(".color1{ background:" + get_color(1) + ";}")

function readInputs() {
    method = document.getElementById('method').value;
    tempSteps = parseInt(document.getElementById('tempSteps').value);
    startTemp = parseFloat(document.getElementById('startTemp').value);
    eSteps = parseInt(document.getElementById('eSteps').value);
    mSteps = parseInt(document.getElementById('mSteps').value);
    energies = make_energies(
        parseFloat(document.getElementById("j00").value),
        parseFloat(document.getElementById("j01").value),
        parseFloat(document.getElementById("j11").value),
    )
    cA = parseFloat(document.getElementById("cA").value);
    cB = parseFloat(document.getElementById("cB").value);
    switch (parseInt(document.getElementById('modelSize').value)) {
        case 16:
            Model = XSmallModel;
            break;
        case 32:
            Model = SmallModel;
            break;
        case 64:
            Model = MediumModel;
            break;
        case 128:
            Model = BigModel;
            break;
        case 256:
            Model = XBigModel;
            break;
        default:
            console.log("failed to assign model, using SmallModel");
            console.log("modelSize: " + modelSize);
            Model = SmallModel
    }
}

function plot(id, xs, ys, x_title, y_title) {
    const trace1 = {
        x: xs,
        y: ys,
    };

    const l1 = {
        ...PLOTLY_LAYOUT,
        xaxis: {
            ...PLOTLY_LAYOUT.xaxis,
            title: x_title,
        },
        yaxis: {
            ...PLOTLY_LAYOUT.yaxis,
            title: y_title,
        },
    };
    Plotly.newPlot(id, [trace1], l1);
}

function setUiRunning() {
    requestAnimationFrame(() => {
        gsap.to("#run", { disabled: true, duration: 0 })
        gsap.to("#modelOutput", { display: "none", duration: 0 })
        gsap.to("#running", { display: "block", duration: 0 })
        gsap.to("#progress", { width: "0%", duration: 0 })
    })
}
function setUiOutput() {
    requestAnimationFrame(() => {
        gsap.to("#run", { disabled: false, innerHTML: "Rerun", duration: 0.01 })
        gsap.to("#modelOutput", { display: "block", duration: 0.01 })
        gsap.to("#running", { display: "none", duration: 0.02 })
    })
}

function runSimulation() {
    model = undefined;
    readInputs();
    setUiRunning()
    nFrames = Math.round(GIF_DURATION / tempSteps / 0.1); // for around 10fps
    let sPerFrame = GIF_DURATION / tempSteps / nFrames;
    model = Model.new(energies, cA, cB, method, tempSteps, Math.round(sPerFrame * 100));

    function animateSimulation(i) {
        if (i < tempSteps) {
            const temp = startTemp * ((tempSteps - 1 - i) / (tempSteps - 1));
            model.run_at_temp(eSteps, mSteps, temp, nFrames);
            gsap.to("#progress", { duration: 0, width: (i / (tempSteps - 1)) * 100 + "%" });
            i++;
            requestAnimationFrame(() => animateSimulation(i));
        } else {
            model.do_data_analysis();
            console.log("simulation done")
            setGif();
            makePlots();
            setUiOutput();
        }
    }
    console.log("simulation started")
    animateSimulation(0)
}

function setGif() {
    const gifLen = model.gif_len();
    const gifPtr = model.gif_ptr();
    const gifData = new Uint8Array(memory.buffer, gifPtr, gifLen);
    let blob = new Blob([gifData], { type: "image/gif" });
    let url = URL.createObjectURL(blob);
    let img = document.getElementById("animationGif");
    img.src = url;
}

function makePlots() {
    const temp = new Float32Array(memory.buffer, model.temp_ptr(), model.log_len());
    const energy = new Float32Array(memory.buffer, model.int_energy_ptr(), model.log_len());
    const heat_capacity = new Float32Array(memory.buffer, model.heat_capacity_ptr(), model.log_len());
    const entropy = new Float32Array(memory.buffer, model.entropy_ptr(), model.log_len());
    const free_energy = new Float32Array(memory.buffer, model.free_energy_ptr(), model.log_len());
    const acceptance = new Float32Array(memory.buffer, model.acceptance_rate_ptr(), model.log_len());

    plot("energyTemp", temp, energy, "Temperature", "Energy (pL)");
    plot("capacityTemp", temp, heat_capacity, "Temperature", "Heat Capacity (pL)");
    plot("acceptanceTemp", temp, acceptance, "Temperature", "Acceptance Rate");
    plot("entropyTemp", temp, entropy, "Temperature", "Entropy (pL)");
    plot("freeTemp", temp, free_energy, "Temperature", "Free Energy (pL)");
}

function getZip() {
    model.make_zip(method, tempSteps, startTemp, eSteps, mSteps);
    const ptr = model.get_zip_ptr();
    const len = model.get_zip_len();
    const zipData = new Uint8Array(memory.buffer, ptr, len);
    const blob = new Blob([zipData], { type: 'applications/zip' });
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = "thermodynamic_model.zip";
    link.click();

    URL.revokeObjectURL(url)
    model.destory_zip_data()
}


window.runSimulation = runSimulation;
window.getZip = getZip