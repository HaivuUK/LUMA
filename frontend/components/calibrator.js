import * as THREE from 'three';
import { TransformControls } from 'three/addons/controls/TransformControls.js';
const invoke = window.__TAURI__.core.invoke;

export class CalibrationManager {
    constructor(scene, camera, renderer, arcballControls) {
        this.scene = scene;
        this.camera = camera;
        this.renderer = renderer;
        this.arcballControls = arcballControls;

        this.rois = [];
        this.nextId = 0;
        this.selectRoiId = null;

        this.transformControls = new TransformControls(this.camera, this.renderer.domElement);
        this.scene.add(this.transformControls.getHelper());

        this.container = document.getElementById('roi-list-container');
        this.btnAddRoi = document.getElementById('btn-add-roi');
        this.btnCalculate = document.getElementById('btn-calibration-calculate') || document.getElementById('btn-calculate');
        this.btnSaveToml = document.getElementById('btn-save-calibration') || document.getElementById('btn-save-toml');
        this.slopeOutput = document.getElementById('roi-slope-value') || document.getElementById('output-slope');
        this.interceptOutput = document.getElementById('roi-intercept-value') || document.getElementById('output-intercept');
        this.template = document.getElementById('roi-row-template');

        this.initEventListeners();
    }

    initEventListeners() {
        this.btnAddRoi?.addEventListener('click', () => this.addRoi());
        this.btnCalculate?.addEventListener('click', () => this.calculateCalibration());
        this.btnSaveToml?.addEventListener('click', () => this.saveCalibrationToToml());

        this.transformControls.addEventListener('change', () => {
            if (this.transformControls.object && this.selectRoiId !== null) {
                this.syncUIFrom3D(this.selectRoiId, this.transformControls.object);
            }
        });

        this.transformControls.addEventListener('dragging-changed', (event) => {
            if (this.arcballControls) {
                this.arcballControls.enabled = !event.value;
            }
        });
    }

    triggerRedraw() {
        window.dispatchEvent(new CustomEvent('calibration-roi-updated'));
    }

    addRoi(initialData = null) {
        const id = `roi_${this.nextId++}`;

        const colours = [
            { hex: 0x00ff00, css: '#00ff00' }, // Green
            { hex: 0xff00ff, css: '#ff00ff' }, // Magenta
            { hex: 0x00ffff, css: '#00ffff' }, // Cyan
            { hex: 0xffcc00, css: '#ffcc00' }, // Yellow
            { hex: 0xff3333, css: '#ff3333' }, // Red
            { hex: 0x3366ff, css: '#3366ff' }, // Blue
            { hex: 0xccff00, css: '#ccff00' }  // Lime
        ];

        const uiColour = colours[(this.nextId - 1) % colours.length];

        const roi = {
            id: id,
            name: initialData?.name || `ROI ${this.nextId}`,
            targetBMD: initialData?.targetBMD ?? 0.0,
            meanHU: 0.0,
            x: initialData?.x || { start: 10, end: 20 },
            y: initialData?.y || { start: 10, end: 20 },
            z: initialData?.z || { start: 10, end: 20 },
            mesh: null,
            colorHex: uiColour.hex,
            colorCss: uiColour.css
        };

        const clone = this.template.content.cloneNode(true);
        const rowEl = clone.querySelector('.roi-row');
        rowEl.setAttribute('data-id', id);

        rowEl.dataset.expanded = 'true';

        rowEl.querySelector('.roi-name').value = roi.name;
        rowEl.querySelector('.roi-bmd').value = roi.targetBMD;
        rowEl.querySelector('.slice-start-x').value = roi.x.start;
        rowEl.querySelector('.slice-end-x').value = roi.x.end;
        rowEl.querySelector('.slice-start-y').value = roi.y.start;
        rowEl.querySelector('.slice-end-y').value = roi.y.end;
        rowEl.querySelector('.slice-start-z').value = roi.z.start;
        rowEl.querySelector('.slice-end-z').value = roi.z.end;

        rowEl.style.borderLeft = `4px solid ${roi.colorCss}`;

        this.container.appendChild(rowEl);
        this.rois.push(roi);

        this.bindRowEvents(rowEl, roi);

        this.create3DBox(roi);

        this.selectROI(roi.id);

        roi.samplePromise = this.sampleVoxelData(roi);
        this.triggerRedraw();
    }

    bindRowEvents(rowEl, roi) {
        const updateState = () => {
            roi.name = rowEl.querySelector('.roi-name').value;
            roi.targetBMD = parseFloat(rowEl.querySelector('.roi-bmd').value) || 0.0;

            roi.x.start = parseInt(rowEl.querySelector('.slice-start-x').value) || 0;
            roi.x.end = parseInt(rowEl.querySelector('.slice-end-x').value) || 0;
            roi.y.start = parseInt(rowEl.querySelector('.slice-start-y').value) || 0;
            roi.y.end = parseInt(rowEl.querySelector('.slice-end-y').value) || 0;
            roi.z.start = parseInt(rowEl.querySelector('.slice-start-z').value) || 0;
            roi.z.end = parseInt(rowEl.querySelector('.slice-end-z').value) || 0;

            this.update3DBoxVisual(roi);
            roi.samplePromise = this.sampleVoxelData(roi);
            this.triggerRedraw();
        };

        rowEl.querySelectorAll('input').forEach(input => {
            input.addEventListener('input', updateState);
        });

        const toggleButton = rowEl.querySelector('.roi-toggle');
        if (toggleButton) {
            toggleButton.addEventListener('click', (event) => {
                event.stopPropagation();
                this.toggleRowDetails(rowEl);
            });
        }

        rowEl.addEventListener('click', (e) => {
           if (!e.target.classList.contains('btn-delete-roi')) {
               this.selectROI(roi.id);
           }
        });

        rowEl.querySelector('.btn-delete-roi').addEventListener('click', (event) => {
            event.stopPropagation();
            this.removeROI(roi.id);
        });
    }

    removeROI(id) {
        const index = this.rois.findIndex((r) => r.id === id);
        if (index !== -1) {
            const roi = this.rois[index];
            if (roi.mesh) {
                if (this.transformControls.object === roi.mesh) {
                    this.transformControls.detach();
                }
                this.scene.remove(roi.mesh);
                roi.mesh.geometry.dispose();
                roi.mesh.material.dispose();
            }
            this.rois.splice(index, 1);
        }

        const rowEl = this.container.querySelector(`[data-id="${id}"]`);
        if (rowEl) rowEl.remove();

        if (this.selectRoiId === id) {
            this.selectRoiId = null;
        }
        this.triggerRedraw();
    }

    selectROI(id) {
        this.selectRoiId = id;

        this.container.querySelectorAll('.roi-row').forEach(row => {
            if (row.getAttribute('data-id') === id) {
                row.style.borderTopColor = '#00ffcc';
                row.style.borderRightColor = '#00ffcc';
                row.style.borderBottomColor = '#00ffcc';
                row.style.background = '#333333';
            } else {
                row.style.borderTopColor = '#444';
                row.style.borderRightColor = '#444';
                row.style.borderBottomColor = '#444';
                row.style.background = '#2a2a2a';
            }
        });

        const roi = this.rois.find(r => r.id === id);
        if (roi && roi.mesh) {
            this.transformControls.attach(roi.mesh);
        }

        const selectedRoi = this.rois.find(r => r.id === id);
        if (selectedRoi) {
            window.dispatchEvent(new CustomEvent('calibration-roi-selected', { detail: { roi: selectedRoi } }));
        }
    }

    create3DBox(roi) {
        const geometry = new THREE.BoxGeometry(1, 1, 1);
        const material = new THREE.MeshBasicMaterial({
            color: roi.colorHex,
            wireframe: true,
            transparent: true,
            opacity: 0.6,
        });

        roi.mesh = new THREE.Mesh(geometry, material);
        roi.mesh.userData = { roiId: roi.id };

        this.scene.add(roi.mesh);
        this.update3DBoxVisual(roi);
    }

    update3DBoxVisual(roi) {
        if (!roi.mesh) return;

        const width = (roi.x.end - roi.x.start + 1);
        const height = (roi.y.end - roi.y.start + 1);
        const depth = (roi.z.end - roi.z.start + 1);

        roi.mesh.scale.set(width, height, depth);

        const posX = ((roi.x.start + roi.x.end) / 2);
        const posY = ((roi.y.start + roi.y.end) / 2);
        const posZ = ((roi.z.start + roi.z.end) / 2);

        roi.mesh.position.set(posX, posY, posZ);
    }

    syncUIFrom3D(id, mesh) {
        const roi = this.rois.find(r => r.id === id);
        if (!roi) return;

        const sizeX = mesh.scale.x;
        const sizeY = mesh.scale.y;
        const sizeZ = mesh.scale.z;

        const halfSlicesX = (sizeX) / 2;
        const halfSlicesY = (sizeY) / 2;
        const halfSlicesZ = (sizeZ) / 2;

        const centerX = mesh.position.x;
        const centerY = mesh.position.y;
        const centerZ = mesh.position.z;

        roi.x.start = Math.round(centerX - halfSlicesX);
        roi.x.end = Math.round(centerX + halfSlicesX - 1);
        roi.y.start = Math.round(centerY - halfSlicesY);
        roi.y.end = Math.round(centerY + halfSlicesY - 1);
        roi.z.start = Math.round(centerZ - halfSlicesZ);
        roi.z.end = Math.round(centerZ + halfSlicesZ - 1);

        if (roi.x.end < roi.x.start) roi.x.end = roi.x.start;
        if (roi.y.end < roi.y.start) roi.y.end = roi.y.start;
        if (roi.z.end < roi.z.start) roi.z.end = roi.z.start;

        const rowEl = this.container.querySelector(`[data-id="${id}"]`);
        if (rowEl) {
            rowEl.querySelector('.slice-start-x').value = roi.x.start;
            rowEl.querySelector('.slice-end-x').value = roi.x.end;
            rowEl.querySelector('.slice-start-y').value = roi.y.start;
            rowEl.querySelector('.slice-end-y').value = roi.y.end;
            rowEl.querySelector('.slice-start-z').value = roi.z.start;
            rowEl.querySelector('.slice-end-z').value = roi.z.end;
        }

        roi.samplePromise = this.sampleVoxelData(roi);
        this.triggerRedraw();
    }

    async sampleVoxelData(roi) {
        const rowEl = this.container.querySelector(`[data-id="${roi.id}"]`);
        const meanHuEl = rowEl?.querySelector('.roi-mean-hu');
        if (meanHuEl) {
            meanHuEl.innerText = '...';
        }

        const requestToken = (roi.sampleToken || 0) + 1;
        roi.sampleToken = requestToken;

        try {
            const meanHU = await invoke('get_ct_roi_mean', {
                xStart: Math.min(roi.x.start, roi.x.end),
                xEnd: Math.max(roi.x.start, roi.x.end),
                yStart: Math.min(roi.y.start, roi.y.end),
                yEnd: Math.max(roi.y.start, roi.y.end),
                zStart: Math.min(roi.z.start, roi.z.end),
                zEnd: Math.max(roi.z.start, roi.z.end)
            });

            if (roi.sampleToken !== requestToken) {
                return;
            }

            roi.meanHU = Number(meanHU);
        } catch (error) {
            if (roi.sampleToken !== requestToken) {
                return;
            }

            console.error(`Failed to sample ROI ${roi.id}:`, error);
            roi.meanHU = NaN;
        }

        if (meanHuEl) {
            meanHuEl.innerText = Number.isFinite(roi.meanHU) ? roi.meanHU.toFixed(2) : '--';
        }
    }

    async calculateCalibration() {
        const pendingSamples = this.rois
            .map(roi => roi.samplePromise)
            .filter(Boolean);

        if (pendingSamples.length > 0) {
            await Promise.allSettled(pendingSamples);
        }

        // Filter out non samples
        const validData = this.rois.filter(r => Number.isFinite(r.meanHU) && Number.isFinite(r.targetBMD));

        if (validData.length < 2) { // need two points to function
            alert("2 samples needed to function")
            this.slope = 0;
            this.intercept = 0;
            if (this.slopeOutput) {
                this.slopeOutput.innerText = '0.0000000000';
            }
            if (this.interceptOutput) {
                this.interceptOutput.innerText = '0.0000000000';
            }
            return {
                slope: 0,
                intercept: 0,
                r2: 0
            };
        }

        let sumX = 0, sumY = 0, sumXY = 0, sumX2 = 0, sumY2 = 0;
        const n = validData.length;

        validData.forEach(roi => {
           const x = roi.meanHU;
           const y = roi.targetBMD;
           sumX += x;
           sumY += y;
           sumXY += (x * y);
           sumX2 += (x * x);
           sumY2 += (y * y);
        });

        const denominator = (n * sumX2 - sumX * sumX);
        if (denominator === 0) {
            alert('Calibration samples must have different HU values');
            return {
                slope: 0,
                intercept: 0,
                r2: 0
            };
        }

        const slope = (n * sumXY - sumX * sumY) / denominator;
        const intercept = (sumY - slope * sumX) / n;

        let residualSum = 0;
        let totalSum = 0;
        const meanY = sumY / n;

        validData.forEach(roi => {
            const predicted = slope * roi.meanHU + intercept;
            residualSum += (roi.targetBMD - predicted) ** 2;
            totalSum += (roi.targetBMD - meanY) ** 2;
        });

        const r2 = totalSum > 0 ? 1 - (residualSum / totalSum) : 0;

        this.slope = slope;
        this.intercept = intercept;
        this.r2 = r2;

        if (this.slopeOutput) {
            this.slopeOutput.innerText = slope.toFixed(10);
        }
        if (this.interceptOutput) {
            this.interceptOutput.innerText = intercept.toFixed(10);
        }

        return {
            slope,
            intercept,
            r2
        };
    }

    saveCalibrationToToml() {
        if (this.slope === undefined || this.intercept === undefined) {
            alert("Calibration data not available");
            return;
        }

        const timestamp = new Date().toISOString();

        let tomlString = `# Calibration data generated on ${timestamp}\n`;
        tomlString += `[ct_calibration_coefficients]\n`;

        // slope and intercept
        tomlString += `rho_qct_a = ${this.slope.toFixed(10)}\n`;
        tomlString += `rho_qct_b = ${this.intercept.toFixed(10)}\n\n`;

        // extra info
        const roiSummary = this.rois.map(r => `${r.name}(HU:${r.meanHU.toFixed(2)}->BMD:${r.targetBMD})`).join(', ');
        tomlString += `# Calculated using points: ${roiSummary}\n`;
        this.rois.forEach(r => {
            tomlString += `#${r.id} = { name = "${r.name}", x = [${r.x.start}, ${r.x.end}], y = [${r.y.start}, ${r.y.end}], z = [${r.z.start}, ${r.z.end}] }\n`;
        });

        triggerDownload(tomlString, `ct_calibration_${timestamp}.toml`);

        function triggerDownload(content, fileName) {
            const blob = new Blob([content], {type: "text/plain"});
            const a = document.createElement("a");
            a.href = URL.createObjectURL(blob);
            a.download = fileName;
            a.click();
        }
    }

    toggleRowDetails(rowEl) {
        const details = rowEl.querySelector('.roi-row-bottom');
        if (!details) {
            return;
        }

        details.hidden = !details.hidden;
        rowEl.dataset.expanded = String(!details.hidden);
    }
}