// scan.js - Camera-based barcode/QR code scanning using zbar-wasm

let videoStream = null;
let scanning = false;
let animationFrameId = null;

// DOM elements
let video = null;
let canvas = null;
let ctx = null;
let startButton = null;
let stopButton = null;
let scanAgainButton = null;
let codeValueInput = null;
let codeValueDisplay = null;
let codeTypeInput = null;
let formSection = null;
let scannerSection = null;
let errorContainer = null;

// Initialize when DOM is loaded
document.addEventListener('DOMContentLoaded', async () => {
    // Get DOM elements
    video = document.getElementById('camera-preview');
    canvas = document.getElementById('capture-canvas');
    ctx = canvas.getContext('2d');
    startButton = document.getElementById('start-scan');
    stopButton = document.getElementById('stop-scan');
    scanAgainButton = document.getElementById('scan-again');
    codeValueInput = document.getElementById('code_value');
    codeValueDisplay = document.getElementById('code_value_display');
    codeTypeInput = document.getElementById('code_type');
    formSection = document.getElementById('form-section');
    scannerSection = document.getElementById('scanner-section');
    errorContainer = document.getElementById('error-container');

    // Setup event listeners
    if (startButton) {
        startButton.addEventListener('click', startScanning);
    }
    if (stopButton) {
        stopButton.addEventListener('click', stopScanning);
    }
    if (scanAgainButton) {
        scanAgainButton.addEventListener('click', scanAgain);
    }

    // Cleanup on page unload
    window.addEventListener('beforeunload', stopScanning);

    // Check for camera support
    if (!navigator.mediaDevices || !navigator.mediaDevices.getUserMedia) {
        showError('Camera access is not supported in this browser. Please use a modern browser or try manual entry.');
        if (startButton) startButton.disabled = true;
    }
});

async function startScanning() {
    try {
        // Hide any previous errors
        hideError();

        // Request camera access
        const constraints = {
            video: {
                facingMode: { ideal: 'environment' }, // Prefer rear camera on mobile
                width: { ideal: 1280 },
                height: { ideal: 720 }
            }
        };

        videoStream = await navigator.mediaDevices.getUserMedia(constraints);
        video.srcObject = videoStream;

        // Wait for video to be ready
        await new Promise((resolve) => {
            video.onloadedmetadata = () => {
                video.play();
                resolve();
            };
        });

        // Setup canvas size to match video
        canvas.width = video.videoWidth;
        canvas.height = video.videoHeight;

        // Update UI
        startButton.style.display = 'none';
        stopButton.style.display = 'inline-block';
        video.style.border = '3px solid var(--color-blue)';

        // Start scanning loop
        scanning = true;
        scanFrame();

    } catch (error) {
        console.error('Error starting camera:', error);
        if (error.name === 'NotAllowedError' || error.name === 'PermissionDeniedError') {
            showError('Camera permission denied. Please allow camera access in your browser settings.');
        } else if (error.name === 'NotFoundError') {
            showError('No camera found. Please connect a camera or use manual entry.');
        } else {
            showError('Failed to access camera: ' + error.message);
        }
    }
}

function stopScanning() {
    scanning = false;

    if (animationFrameId) {
        cancelAnimationFrame(animationFrameId);
        animationFrameId = null;
    }

    if (videoStream) {
        videoStream.getTracks().forEach(track => track.stop());
        videoStream = null;
    }

    if (video) {
        video.srcObject = null;
        video.style.border = 'none';
    }

    if (startButton && stopButton) {
        startButton.style.display = 'inline-block';
        stopButton.style.display = 'none';
    }
}

async function scanFrame() {
    if (!scanning) return;

    try {
        // Capture current video frame to canvas
        ctx.drawImage(video, 0, 0, canvas.width, canvas.height);

        // Get image data
        const imageData = ctx.getImageData(0, 0, canvas.width, canvas.height);

        // Scan for barcodes/QR codes using zbar-wasm
        if (window.scanImageData) {
            const symbols = await window.scanImageData(imageData);

            if (symbols && symbols.length > 0) {
                // Code detected!
                const symbol = symbols[0]; // Take first detected code
                await handleCodeDetected(symbol);
                return; // Stop scanning
            }
        }

    } catch (error) {
        console.error('Error scanning frame:', error);
        // Continue scanning despite errors
    }

    // Schedule next frame scan
    animationFrameId = requestAnimationFrame(scanFrame);
}

async function handleCodeDetected(symbol) {
    try {
        console.log('handleCodeDetected called with symbol:', symbol);

        // Stop scanning
        scanning = false;
        if (animationFrameId) {
            cancelAnimationFrame(animationFrameId);
        }

        // Visual feedback
        video.style.border = '3px solid var(--color-green)';

        // Haptic feedback (if supported)
        if (navigator.vibrate) {
            navigator.vibrate(200);
        }

        // Extract code value - try different methods
        let codeValue;
        if (typeof symbol.decode === 'function') {
            codeValue = symbol.decode();
        } else if (symbol.value) {
            codeValue = symbol.value;
        } else if (symbol.data) {
            codeValue = symbol.data;
        } else {
            console.error('Cannot find decoded value in symbol:', Object.keys(symbol));
            codeValue = String(symbol);
        }

        const typeName = symbol.typeName || symbol.type || 'Unknown';

        console.log('Extracted data:', { codeValue, typeName });

        // Determine code type
        const codeType = determineCodeType(typeName);

        // Populate form
        if (codeValueInput) {
            codeValueInput.value = codeValue;
            console.log('Set codeValueInput.value to:', codeValue);
        } else {
            console.error('codeValueInput not found');
        }

        if (codeValueDisplay) {
            codeValueDisplay.textContent = codeValue;
            console.log('Set codeValueDisplay.textContent to:', codeValue);
        } else {
            console.error('codeValueDisplay not found');
        }

        if (codeTypeInput) {
            codeTypeInput.value = codeType;
            console.log('Set codeTypeInput.value to:', codeType);
        } else {
            console.error('codeTypeInput not found');
        }

        // Show form, hide scanner controls
        if (formSection) {
            formSection.style.display = 'block';
            console.log('Showing form section');
        } else {
            console.error('formSection not found');
        }

        if (stopButton) {
            stopButton.style.display = 'none';
            console.log('Hiding stop button');
        } else {
            console.error('stopButton not found');
        }

        console.log('Code detected and processed:', { typeName, codeType, codeValue });
    } catch (error) {
        console.error('Error in handleCodeDetected:', error);
        showError('Error processing scanned code: ' + error.message);
    }
}

function determineCodeType(typeName) {
    // typeName examples: "QR-Code", "EAN-13", "Code-128", etc.
    if (typeName === 'QR-Code' || typeName == "ZBAR_QRCODE") {
        return 'qrcode';
    } else {
        // All other formats are barcodes
        return 'barcode';
    }
}

function scanAgain() {
    // Reset form
    if (codeValueInput) codeValueInput.value = '';
    if (codeValueDisplay) codeValueDisplay.textContent = '';
    if (codeTypeInput) codeTypeInput.value = '';

    // Hide form
    if (formSection) formSection.style.display = 'none';

    // Reset video border
    if (video) video.style.border = '3px solid var(--color-blue)';

    // Show stop button
    if (stopButton) stopButton.style.display = 'inline-block';

    // Resume scanning
    scanning = true;
    scanFrame();
}

function showError(message) {
    if (errorContainer) {
        errorContainer.textContent = message;
        errorContainer.style.display = 'block';
    }
}

function hideError() {
    if (errorContainer) {
        errorContainer.style.display = 'none';
    }
}
