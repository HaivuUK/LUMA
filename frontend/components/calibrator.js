import * as THREE from 'three';
import { TransformControls } from 'three/addons/controls/TransformControls.js';
const invoke = window.__TAURI__.core.invoke;

export class CalibrationManager {
    constructor(scene, camera, renderer, arcballControls, getDefaultBounds = null) {
        this.scene = scene;
        this.camera = camera;
        this.renderer = renderer;
        this.arcballControls = arcballControls;
        this.getDefaultBounds = getDefaultBounds;

        this.rois = [];
        this.nextId = 0;
        this.selectRoiId = null;

        this.transformControls = new TransformControls(this.camera, this.renderer.domElement);
        this.scene.add(this.transformControls.getHelper());

        // raycasting for nodal roi
        this.raycaster = new THREE.Raycaster();
        this.raycaster.params.Line.threshold = 1;
        this.pointer = new THREE.Vector2();
        this.draggingNode = null;

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

        // nodal roi
        const dom = this.renderer.domElement;
        dom.addEventListener('pointerdown', (event) => this.onNodalPointerDown(event));
        dom.addEventListener('pointermove', (event) => this.onNodalPointerMove(event));
        window.addEventListener('pointerup', () => this.onNodalPointerUp());
        dom.addEventListener('dblclick', (event) => this.onNodalDoubleClick(event));
    }

    triggerRedraw() {
        this.rois.forEach(roi => this.updateROIVisual(roi));
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
        const shape = initialData?.shape || 'box';

        const roi = {
            id: id,
            name: initialData?.name || `ROI ${this.nextId}`,
            targetBMD: initialData?.targetBMD ?? 0.0,
            meanHU: 0.0,
            shape: shape,
            mesh: null,
            group: null,
            edgeLines: null,
            nodeHandles: [],
            colorHex: uiColour.hex,
            colorCss: uiColour.css
        };

        this.initShapeData(roi, shape, initialData || {});

        const clone = this.template.content.cloneNode(true);
        const rowEl = clone.querySelector('.roi-row');
        rowEl.setAttribute('data-id', id);

        rowEl.dataset.expanded = 'true';

        rowEl.querySelector('.roi-name').value = roi.name;
        rowEl.querySelector('.roi-bmd').value = roi.targetBMD;
        rowEl.querySelector('.roi-shape-select').value = roi.shape;

        rowEl.style.borderLeft = `4px solid ${roi.colorCss}`;

        this.populateShapeFields(rowEl, roi);
        this.updateShapeFieldVisibility(rowEl, roi.shape);

        this.container.appendChild(rowEl);
        this.rois.push(roi);

        this.bindRowEvents(rowEl, roi);

        this.createROIMesh(roi);

        this.selectROI(roi.id);

        roi.samplePromise = this.sampleVoxelData(roi);
        this.triggerRedraw();
    }

    initShapeData(roi, shape, initialData = null) {
        const defaults = (!initialData?.x && this.getDefaultBounds) ? this.getDefaultBounds() : null; // get default bounds
        const z = initialData?.z || defaults?.z || { start: 10, end: 20 };

        if (shape === 'cylinder') {
            roi.cx = initialData?.cx || (defaults ? (defaults.x.start + defaults.x.end) / 2 : 15);
            roi.cy = initialData?.cy || (defaults ? (defaults.y.start + defaults.y.end) / 2 : 15);
            roi.radius = initialData?.radius || defaults?.radius || 5;
            roi.z = { ...z };
        } else if (shape === 'nodal') {
            roi.nodes = initialData?.nodes || [
                { x: defaults?.x.start, y: defaults?.y.start} || { x: 10, y: 10},
                { x: defaults?.x.end, y: defaults?.y.start} || { x: 20, y: 10},
                { x: defaults?.x.end, y: defaults?.y.end} || { x: 20, y: 20},
                { x: defaults?.x.start, y: defaults?.y.end} || { x: 10, y: 20}
            ];
            roi.z = { ...z };
        } else {
            roi.x = initialData?.x || defaults?.x || { start: 10, end: 20 };
            roi.y = initialData?.y || defaults?.y || { start: 10, end: 20 };
            roi.z = { ...z };
        }
    }

    changeROIShape(roi, newShape, rowEl) {
        if (roi.shape === newShape) return;

        if (this.transformControls.object === roi.mesh) {
            this.transformControls.detach();
        }
        if (this.draggingNode && this.draggingNode.roi.id === roi.id) {
            this.draggingNode = null;
        }
        this.disposeROIMesh(roi);

        const zRange = roi.z ? { ...roi.z } : { start: 10, end: 20 };
        roi.shape = newShape;
        this.initShapeData(roi, newShape, { z: zRange });

        this.populateShapeFields(rowEl, roi);
        this.updateShapeFieldVisibility(rowEl, newShape);

        this.createROIMesh(roi);

        if (this.selectRoiId === roi.id) {
            this.selectROI(roi.id);
        }

        roi.samplePromise = this.sampleVoxelData(roi);
        this.triggerRedraw();
    }

    bindRowEvents(rowEl, roi) {
        const updateCommonState = () => {
            roi.name = rowEl.querySelector('.roi-name').value;
            roi.targetBMD = parseFloat(rowEl.querySelector('.roi-bmd').value) || 0.0;
            this.triggerRedraw();
        };

        const updateShapeState = () => {
            if (roi.shape === 'box') {
                roi.x.start = parseInt(rowEl.querySelector('.slice-start-x').value) || 0;
                roi.x.end = parseInt(rowEl.querySelector('.slice-end-x').value) || 0;
                roi.y.start = parseInt(rowEl.querySelector('.slice-start-y').value) || 0;
                roi.y.end = parseInt(rowEl.querySelector('.slice-end-y').value) || 0;
                roi.z.start = parseInt(rowEl.querySelector('.slice-start-z').value) || 0;
                roi.z.end = parseInt(rowEl.querySelector('.slice-end-z').value) || 0;
            } else if (roi.shape === 'cylinder') {
                roi.cx = parseFloat(rowEl.querySelector('.cyl-cx').value) || 0;
                roi.cy = parseFloat(rowEl.querySelector('.cyl-cy').value) || 0;
                roi.radius = Math.max(0.5, parseFloat(rowEl.querySelector('.cyl-radius').value) || 0.5);
                roi.z.start = parseInt(rowEl.querySelector('.cyl-start-z').value) || 0;
                roi.z.end = parseInt(rowEl.querySelector('.cyl-end-z').value) || 0;
            } else if (roi.shape === 'nodal') {
                roi.z.start = parseInt(rowEl.querySelector('.nodal-start-z').value) || 0;
                roi.z.end = parseInt(rowEl.querySelector('.nodal-end-z').value) || 0;
            }

            this.updateROIVisual(roi);
            roi.samplePromise = this.sampleVoxelData(roi);
            this.triggerRedraw();
        };

        rowEl.querySelectorAll('.roi-name, .roi-bmd').forEach(input => {
            input.addEventListener('input', updateCommonState);
        });

        rowEl.querySelectorAll('.roi-box-fields input, .roi-cylinder-fields input, .roi-nodal-fields input')
            .forEach(input => input.addEventListener('input', updateShapeState));

        const shapeSelect = rowEl.querySelector('.roi-shape-select');
        shapeSelect?.addEventListener('change', () => {
            this.changeROIShape(roi, shapeSelect.value, rowEl);
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
            if (this.transformControls.object === roi.mesh) {
                this.transformControls.detach();
            }
            if (this.draggingNode && this.draggingNode.roi.id === id) {
                this.draggingNode = null;
            }
            this.disposeROIMesh(roi);
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
        if (roi && (roi.shape === 'box' || roi.shape === 'cylinder') && roi.mesh) {
            this.transformControls.attach(roi.mesh);
        } else {
            this.transformControls.detach();
        }

        if (roi) {
            window.dispatchEvent(new CustomEvent('calibration-roi-selected', { detail: { roi } }));
        }
    }

    populateShapeFields(rowEl, roi) {
        if (roi.shape === 'box') {
            rowEl.querySelector('.slice-start-x').value = roi.x.start;
            rowEl.querySelector('.slice-end-x').value = roi.x.end;
            rowEl.querySelector('.slice-start-y').value = roi.y.start;
            rowEl.querySelector('.slice-end-y').value = roi.y.end;
            rowEl.querySelector('.slice-start-z').value = roi.z.start;
            rowEl.querySelector('.slice-end-z').value = roi.z.end;
        } else if (roi.shape === 'cylinder') {
            rowEl.querySelector('.cyl-cx').value = roi.cx;
            rowEl.querySelector('.cyl-cy').value = roi.cy;
            rowEl.querySelector('.cyl-radius').value = roi.radius;
            rowEl.querySelector('.cyl-start-z').value = roi.z.start;
            rowEl.querySelector('.cyl-end-z').value = roi.z.end;
        } else if (roi.shape === 'nodal') {
            rowEl.querySelector('.nodal-start-z').value = roi.z.start;
            rowEl.querySelector('.nodal-end-z').value = roi.z.end;
            this.refreshNodeCount(roi, rowEl);
        }
    }

    updateShapeFieldVisibility(rowEl, shape) {
        const box = rowEl.querySelector('.roi-box-fields');
        const cyl = rowEl.querySelector('.roi-cylinder-fields');
        const nodal = rowEl.querySelector('.roi-nodal-fields');
        if (box) box.hidden = shape !== 'box';
        if (cyl) cyl.hidden = shape !== 'cylinder';
        if (nodal) nodal.hidden = shape !== 'nodal';
    }

    refreshNodeCount(roi, rowEl = null) {
        const el = rowEl || this.container.querySelector(`[data-id="${roi.id}"]`);
        const countEl = el?.querySelector('.nodal-node-count');
        if (countEl) countEl.textContent = roi.nodes.length;
    }

    createROIMesh(roi) {
        if (roi.shape === 'cylinder') this.createCylinderMesh(roi);
        else if (roi.shape === 'nodal') this.createNodalMesh(roi);
        else this.createBoxMesh(roi);
    }

    createBoxMesh(roi) {
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
        this.updateBoxVisual(roi);
    }

    createCylinderMesh(roi) {
        const geometry = new THREE.CylinderGeometry(0.5, 0.5, 1, 32);
        geometry.rotateX(Math.PI / 2);

        const material = new THREE.MeshBasicMaterial({
            color: roi.colorHex,
            wireframe: true,
            transparent: true,
            opacity: 0.6,
        });

        roi.mesh = new THREE.Mesh(geometry, material);
        roi.mesh.userData = { roiId: roi.id };

        this.scene.add(roi.mesh);
        this.updateCylinderVisual(roi);
    }

    createNodalMesh(roi) {
        roi.group = new THREE.Group();
        roi.group.userData = { roiId: roi.id };

        const solidMaterial = new THREE.MeshBasicMaterial({
            color: roi.colorHex,
            wireframe: true,
            transparent: true,
            opacity: 0.6,
        });
        roi.mesh = new THREE.Mesh(new THREE.BufferGeometry(), solidMaterial);
        roi.mesh.userData = { roiId: roi.id };
        roi.group.add(roi.mesh);

        const lineMaterial = new THREE.LineBasicMaterial({ color: roi.colorHex });
        roi.edgeLines = new THREE.LineSegments(new THREE.BufferGeometry(), lineMaterial);
        roi.group.add(roi.edgeLines);

        roi.nodeHandles = [];

        this.scene.add(roi.group);
        this.rebuildNodalGeometry(roi);
    }

    updateROIVisual(roi) {
        if (roi.shape === 'cylinder') this.updateCylinderVisual(roi);
        else if (roi.shape === 'nodal') this.rebuildNodalGeometry(roi);
        else this.updateBoxVisual(roi);
    }

    updateBoxVisual(roi) {
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

    updateCylinderVisual(roi) {
        if (!roi.mesh) return;

        const depth = (roi.z.end - roi.z.start + 1);
        const diameter = roi.radius * 2;

        roi.mesh.scale.set(diameter, diameter, depth);
        roi.mesh.position.set(roi.cx, roi.cy, (roi.z.start + roi.z.end) / 2);
    }

    rebuildNodalGeometry(roi) {
        if (!roi.mesh || roi.nodes.length < 3) return;

        const zStart = Math.min(roi.z.start, roi.z.end);
        const zEnd = Math.max(roi.z.start, roi.z.end);
        const depth = Math.max(1, zEnd - zStart + 1);
        const refZ = (zStart + zEnd) /2;

        const shape = new THREE.Shape();
        roi.nodes.forEach((node, i) => {
            if (i === 0) shape.moveTo(node.x, node.y);
            else shape.lineTo(node.x, node.y);
        });
        shape.closePath();

        roi.mesh.geometry.dispose();
        roi.mesh.geometry = new THREE.ExtrudeGeometry(shape, { depth, bevelEnabled: false, curveSegments: 24 });
        roi.mesh.position.set(0, 0, zStart);

        const linePoints = roi.nodes.map(node => new THREE.Vector3(node.x, node.y, refZ));
        roi.edgeLines.geometry.dispose();
        roi.edgeLines.geometry = new THREE.BufferGeometry().setFromPoints(linePoints);

        while (roi.nodeHandles.length < roi.nodes.length) {
            const handleGeom = new THREE.SphereGeometry(0.6, 12, 12);
            const handleMat = new THREE.MeshBasicMaterial({ color: 0xffffff });
            const handle = new THREE.Mesh(handleGeom, handleMat);
            roi.group.add(handle);
            roi.nodeHandles.push(handle);
        }
        while (roi.nodeHandles.length > roi.nodes.length) {
            const handle = roi.nodeHandles.pop();
            roi.group.remove(handle);
            handle.geometry.dispose();
            handle.material.dispose();
        }
        roi.nodeHandles.forEach((handle, i) => {
            handle.position.set(roi.nodes[i].x, roi.nodes[i].y, refZ);
            handle.userData = { roiId: roi.id, nodeIndex: i };
        });
    }

    disposeROIMesh(roi) {
        if (roi.group) {
            this.scene.remove(roi.group);
        } else if (roi.mesh) {
            this.scene.remove(roi.mesh);
        }

        if (roi.mesh) {
            roi.mesh.geometry?.dispose();
            roi.mesh.material?.dispose();
        }
        if (roi.edgeLines) {
            roi.edgeLines.geometry?.dispose();
            roi.edgeLines.material?.dispose();
        }
        (roi.nodeHandles || []).forEach(handle => {
            handle.geometry.dispose();
            handle.material.dispose();
        });

        roi.nodeHandles = [];
        roi.group = null;
        roi.mesh = null;
        roi.edgeLines = null;
    }

    syncUIFrom3D(id, mesh) {
        const roi = this.rois.find(r => r.id === id);
        if (!roi) return;

        const rowEl = this.container.querySelector(`[data-id="${id}"]`);

        if (roi.shape === 'cylinder') {
            roi.cx = mesh.position.x;
            roi.cy = mesh.position.y;
            roi.radius = Math.max(0.5, mesh.scale.x / 2);

            const halfZ = mesh.scale.z / 2;
            roi.z.start = Math.round(mesh.position.z - halfZ);
            roi.z.end = Math.round(mesh.position.z + halfZ - 1);
            if (roi.z.end < roi.z.start) roi.z.end = roi.z.start;

            if (rowEl) {
                rowEl.querySelector('.cyl-cx').value = roi.cx.toFixed(2);
                rowEl.querySelector('.cyl-cy').value = roi.cy.toFixed(2);
                rowEl.querySelector('.cyl-radius').value = roi.radius.toFixed(2);
                rowEl.querySelector('.cyl-start-z').value = roi.z.start;
                rowEl.querySelector('.cyl-end-z').value = roi.z.end;
            }
        } else {
            const halfX = mesh.scale.x / 2;
            const halfY = mesh.scale.y / 2;
            const halfZ = mesh.scale.z / 2;

            const centerX = mesh.position.x;
            const centerY = mesh.position.y;
            const centerZ = mesh.position.z;

            roi.x.start = Math.round(centerX - halfX);
            roi.x.end = Math.round(centerX + halfX - 1);
            roi.y.start = Math.round(centerY - halfY);
            roi.y.end = Math.round(centerY + halfY - 1);
            roi.z.start = Math.round(centerZ - halfZ);
            roi.z.end = Math.round(centerZ + halfZ - 1);

            if (roi.x.end < roi.x.start) roi.x.end = roi.x.start;
            if (roi.y.end < roi.y.start) roi.y.end = roi.y.start;
            if (roi.z.end < roi.z.start) roi.z.end = roi.z.start;

            if (rowEl) {
                rowEl.querySelector('.slice-start-x').value = roi.x.start;
                rowEl.querySelector('.slice-end-x').value = roi.x.end;
                rowEl.querySelector('.slice-start-y').value = roi.y.start;
                rowEl.querySelector('.slice-end-y').value = roi.y.end;
                rowEl.querySelector('.slice-start-z').value = roi.z.start;
                rowEl.querySelector('.slice-end-z').value = roi.z.end;
            }
        }

        roi.samplePromise = this.sampleVoxelData(roi);
        this.triggerRedraw();
    }

    updatePointerFromEvent(event) {
        const rect = this.renderer.domElement.getBoundingClientRect();
        this.pointer.x = ((event.clientX - rect.left) / rect.width) * 2 - 1;
        this.pointer.y = -((event.clientY - rect.top) / rect.height) * 2 + 1;
    }

    getSelectedNodalROI() {
        const roi = this.rois.find(r => r.id === this.selectRoiId);
        return (roi && roi.shape === 'nodal') ? roi : null;
    }

    getNodalDragPlane(roi) {
        const zStart = Math.min(roi.z.start, roi.z.end);
        const zEnd = Math.max(roi.z.start, roi.z.end);
        const refZ = (zStart + zEnd) / 2;
        return new THREE.Plane(new THREE.Vector3(0, 0, 1), -refZ);
    }

    onNodalPointerDown(event) {
        const roi = this.getSelectedNodalROI();
        if (!roi) return;

        this.updatePointerFromEvent(event);
        this.raycaster.setFromCamera(this.pointer, this.camera);

        const nodeHits = this.raycaster.intersectObjects(roi.nodeHandles);

        if (nodeHits.length > 0) {
            this.draggingNode = { roi, mode: 'node', nodeIndex: nodeHits[0].object.userData.nodeIndex };
            if (this.arcballControls) this.arcballControls.enabled = false;
            event.stopPropagation();
            return;
        }

        const bodyHits = this.raycaster.intersectObject(roi.mesh);
        if (bodyHits.length > 0) {
            this.draggingNode = { roi, mode: 'body', lastPoint: bodyHits[0].point.clone() };
            if (this.arcballControls) this.arcballControls.enabled = false;
            event.stopPropagation();
        }
    }

    onNodalPointerMove(event) {
        if (!this.draggingNode) return;
        const drag = this.draggingNode;
        const roi = drag.roi;

        this.updatePointerFromEvent(event);
        this.raycaster.setFromCamera(this.pointer, this.camera);

        const plane = this.getNodalDragPlane(roi);
        const point = new THREE.Vector3();
        if (this.raycaster.ray.intersectPlane(plane, point)) return;

        if (drag.mode === 'node') {
            roi.nodes[drag.nodeIndex].x = point.x;
            roi.nodes[drag.nodeIndex].y = point.y;
        } else {
            const dx = point.x - drag.lastPoint.x;
            const dy = point.y - drag.lastPoint.y;
            roi.nodes.forEach(node => { node.x += dx; node.y += dy; });
            drag.lastPoint.copy(point);
        }

        this.rebuildNodalGeometry(roi);
        this.triggerRedraw();
    }

    onNodalPointerUp() {
        if (!this.draggingNode) return;
        const { roi } = this.draggingNode;
        this.draggingNode = null;
        if (this.arcballControls) this.arcballControls.enabled = true;
        roi.samplePromise = this.sampleVoxelData(roi);
    }

    onNodalDoubleClick(event) {
        const roi = this.getSelectedNodalROI();
        if (!roi) return;

        this.updatePointerFromEvent(event);
        this.raycaster.setFromCamera(this.pointer, this.camera);

        // delete nodes
        const handleHits = this.raycaster.intersectObjects(roi.nodeHandles);
        if (handleHits.length > 0) {
            const nodeIndex = handleHits[0].object.userData.nodeIndex;
            if (roi.nodes.length > 3) {
                roi.nodes.splice(nodeIndex, 1);
                this.rebuildNodalGeometry(roi);
                roi.samplePromise = this.sampleVoxelData(roi);
                this.triggerRedraw();
                this.refreshNodeCount(roi);
            }
            event.stopPropagation();
            return;
        }

        // make nodes
        const edgeHits = this.raycaster.intersectObjects(roi.edgeLines);
        if (edgeHits.length > 0) {
            const hitPoint = edgeHits[0].point;
            const insertIndex = this.findClosestEdgeIndex(roi, hitPoint);
            roi.nodes.splice(insertIndex, 0, { x: hitPoint.x, y: hitPoint.y });
            this.rebuildNodalGeometry(roi);
            roi.samplePromise = this.sampleVoxelData(roi);
            this.triggerRedraw();
            this.refreshNodeCount(roi);
            event.stopPropagation();
        }
    }

    findClosestEdgeIndex(roi, point) {
        let bestIndex = roi.nodes.length;
        let bestDist = Infinity;

        for (let i = 0; i < roi.nodes.length; i++) {
            const a = roi.nodes[i];
            const b = roi.nodes[(i + 1) % roi.nodes.length];
            const dist = this.pointToSegmentDistance(point, a, b);
            if (dist < bestDist) {
                bestDist = dist;
                bestIndex = i + 1;
            }
        }
        return bestIndex;
    }

    pointToSegmentDistance(point, a, b) {
        const abx = b.x - a.x, aby = b.y - a.y;
        const apx = point.x - a.x, apy = point.y - a.y;
        const lenSq = abx * abx + aby * aby;
        let t = lenSq > 0 ? (apx * abx + apy * aby) / lenSq : 0;
        t = Math.max(0, Math.min(1, t));
        const cx = a.x + abx * t, cy = a.y + aby * t;
        return Math.hypot(point.x - cx, point.y - cy);
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
            let meanHU;

            if (roi.shape === 'cylinder') {
                meanHU = await invoke('get_ct_roi_mean_cylinder', {
                    centerX: roi.cx,
                    centerY: roi.cy,
                    radius: roi.radius,
                    zStart: Math.min(roi.z.start, roi.z.end),
                    zEnd: Math.max(roi.z.start, roi.z.end),
                });
            } else if (roi.shape === 'nodal') {
                meanHU = await invoke('get_ct_roi_mean_polygon', {
                    points: roi.nodes.map(node => [node.x, node.y]),
                    zStart: Math.min(roi.z.start, roi.z.end),
                    zEnd: Math.max(roi.z.start, roi.z.end)
                });
            } else {
                meanHU = await invoke('get_ct_roi_mean', {
                    xStart: Math.min(roi.x.start, roi.x.end),
                    xEnd: Math.max(roi.x.start, roi.x.end),
                    yStart: Math.min(roi.y.start, roi.y.end),
                    yEnd: Math.max(roi.y.start, roi.y.end),
                    zStart: Math.min(roi.z.start, roi.z.end),
                    zEnd: Math.max(roi.z.start, roi.z.end)
                });
            }

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

        const timestamp = new Date().toISOString().split('.')[0] + 'Z';

        let tomlString = `# Calibration data generated on ${timestamp}\n`;
        tomlString += `[ct_calibration_coefficients]\n`;

        // slope and intercept
        tomlString += `rho_qct_a = ${(this.intercept / 1000).toFixed(10)}\n`;
        tomlString += `rho_qct_b = ${(this.slope / 1000).toFixed(10)}\n\n`;

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

    create3DBox(roi) {
        this.createROIMesh(roi);
    }

    update3DBoxVisual(roi) {
        this.updateROIVisual(roi);
    }
}