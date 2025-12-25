// here's where we put all our shenanigans.

// when the page loads, render the barcode
document.addEventListener("DOMContentLoaded", () => {
	// find the barcode svg and get the data-value attribute
	const barcodeElement = document.querySelector("#barcode");
	if (barcodeElement) {
		const barcodeValue = barcodeElement.getAttribute("data-value");
		if (!barcodeValue) {
			console.error("No barcode value found!");
			return;
		}
		JsBarcode("#barcode", barcodeValue,{
			format: "code128",
			lineColor: "#000",
			displayValue: false
		});
	} else {
		console.debug("No barcode element found.");
	}

	const qrElement = document.querySelector("#qrcode");
	if (qrElement) {
		const qrValue = qrElement.getAttribute("data-value");
		if (!qrValue) {
			console.error("No QR code value found!");
			return;
		} else {
			console.debug("QR code value:", qrValue);
		}
		new QRCode(qrElement, {
			text: qrValue,
			width: 512,
			height: 512,
			colorDark: "#000000",
			colorLight: "#ffffff",
			correctLevel: QRCode.CorrectLevel.H,
		});
	} else {
		console.debug("No QR code element found.");
	}
});
