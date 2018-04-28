function loadImgIntoMem(img) {
	return new Promise(resolve => {
		fetch(img)
			.then(r => r.arrayBuffer())
			.then(buff => {
				const imgUi8buff = new Uint8Array(buff, 0, buff.byteLength);
				resolve({ size: buff.byteLength, imgUi8buff });
			});
	});
}