import { XSmallModel, SmallModel, MediumModel, BigModel, XBigModel, get_color, make_energies } from "thermo-online";
import { memory } from "thermo-online/thermo_online_bg";
import { gsap } from "gsap";
import Plotly from "plotly.js-dist-min";


const GIF_DURATION = 10.0;

let distrPerTemp = 1;
let tempSteps;
let startTemp;
let eSteps;
let mSteps;
let nFrames;
let cA;
let cB;
let method;
let energies;

let ModelType;
let modelInstance;


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
            ModelType = XSmallModel;
            break;
        case 32:
            ModelType = SmallModel;
            break;
        case 64:
            ModelType = MediumModel;
            break;
        case 128:
            ModelType = BigModel;
            break;
        case 256:
            ModelType = XBigModel;
            break;
        default:
            console.log("failed to assign model, using SmallModel");
            console.log("modelSize: " + modelSize);
            ModelType = SmallModel
    }
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
    modelInstance = undefined;
    readInputs();
    setUiRunning();
    nFrames = Math.round(GIF_DURATION / tempSteps / 0.1); // for around 10fps
    let sPerFrame = GIF_DURATION / tempSteps / nFrames;
    modelInstance = ModelType.new(energies, cA, cB, method, tempSteps, Math.round(sPerFrame * 100), distrPerTemp);

    function animateSimulation(i) {
        if (i < tempSteps) {
            const temp = startTemp * ((tempSteps - 1 - i) / (tempSteps - 1));
            modelInstance.run_at_temp(eSteps, mSteps, temp, nFrames, distrPerTemp);
            gsap.to("#progress", { duration: 0, width: (i / (tempSteps - 1)) * 100 + "%" });
            i++;
            requestAnimationFrame(() => animateSimulation(i));
        } else {
            modelInstance.do_data_analysis();
            console.log("simulation done")
            requestAnimationFrame(() => {
                setUiOutput();
                setGif();
            });

            requestAnimationFrame(
                makePlots
            );
        }
    }
    console.log("simulation started")
    animateSimulation(0)
}

function setGif() {
    const gifData = new Uint8Array(memory.buffer, modelInstance.gif_ptr(), modelInstance.gif_len());
    let blob = new Blob([gifData], { type: "image/gif" });
    let url = URL.createObjectURL(blob);
    let img = document.getElementById("animationGif");
    img.src = url;
}

function makePlots() {
    const LINE_COLOR = '#000000';

    const temp = new Float32Array(memory.buffer, modelInstance.temp_ptr(), modelInstance.log_len());
    let data = [];

    function generateAxisLayout(title) {
        return {
            color: '#FFFFFF',
            tickfont: {
                color: '#FFFFFF'
            },
            title: title,
            automargin: true,
        };
    }
    width = document.getElementById("plot").offsetWidth
    console.log(width)

    let layout = {
        autosize: false,
        responsive: true,
        plot_bgcolor: '#555',
        paper_bgcolor: '#444',
        grid: {
            rows: 3,
            columns: 2,
            pattern: 'independent'
        },
        showlegend: false,
        width: width,
        // height: 500,
        // margin: 80
    };

    data.push({
        x: temp,
        y: new Float32Array(memory.buffer, modelInstance.int_energy_ptr(), modelInstance.log_len()),
        line: {
            color: LINE_COLOR
        },
        xaxis: "x",
        yaxis: "y",
    })
    layout.xaxis = generateAxisLayout("Temperature");
    layout.yaxis = generateAxisLayout("Internal Energy");

    data.push({
        x: temp,
        y: new Float32Array(memory.buffer, modelInstance.heat_capacity_ptr(), modelInstance.log_len()),
        xaxis: "x2",
        yaxis: "y2",
        line: {
            color: LINE_COLOR
        }
    })
    layout.xaxis2 = generateAxisLayout("Temperature");
    layout.yaxis2 = generateAxisLayout("Heat Capacity");

    data.push({
        x: temp,
        y: new Float32Array(memory.buffer, modelInstance.acceptance_rate_ptr(), modelInstance.log_len()),
        xaxis: "x3",
        yaxis: "y3",
        line: {
            color: LINE_COLOR
        }
    })
    layout.xaxis3 = generateAxisLayout("Temperature");
    layout.yaxis3 = generateAxisLayout("Acceptance Rate");

    data.push({
        x: temp,
        y: new Float32Array(memory.buffer, modelInstance.entropy_ptr(), modelInstance.log_len()),
        xaxis: "x4",
        yaxis: "y4",
        line: {
            color: LINE_COLOR
        }
    })
    layout.xaxis4 = generateAxisLayout("Temperature");
    layout.yaxis4 = generateAxisLayout("Entropy");

    data.push({
        x: temp,
        y: new Float32Array(memory.buffer, modelInstance.free_energy_ptr(), modelInstance.log_len()),
        xaxis: "x5",
        yaxis: "y5",
        line: {
            color: LINE_COLOR
        }
    })
    layout.xaxis5 = generateAxisLayout("Temperature");
    layout.yaxis5 = generateAxisLayout("Free Energy");

    Plotly.newPlot("plots", data, layout, { responsive: true });
}

function getZip() {
    modelInstance.make_zip(method, tempSteps, startTemp, eSteps, mSteps);
    const zipData = new Uint8Array(memory.buffer, modelInstance.get_zip_ptr(), modelInstance.get_zip_len());
    const blob = new Blob([zipData], { type: 'applications/zip' });
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = "thermodynamic_model.zip";
    link.click();

    URL.revokeObjectURL(url)
    modelInstance.destory_zip_data()
}


window.runSimulation = runSimulation;
window.getZip = getZip