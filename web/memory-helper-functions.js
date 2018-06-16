/**
	Helper function for printing arrays of integers
*/
function printIntArr(ptr, size) {
   let intPtr = ptr >> 2;
   for (let i = intPtr; i < intPtr + size; i++) {
      console.log(Module.HEAP32[i]);
   }
}


/**
	Helper function for printing arrays of integers
*/
function ptrToStr(ptr, size) {
	const HEAP8 = new Int8Array(Module.instance.exports.memory.buffer);

	let str = '';
	for (let i = ptr; i < ptr + size; i++) {
		str += String.fromCharCode(HEAP8[i]);
	}

	return str;
}

function initHeaps() {
	window.heap16 = new Uint32Array(Module.instance.exports.memory.buffer);
	window.heap8 = new Uint8Array(Module.instance.exports.memory.buffer);
}

function printPoint(ptr) {
	let pointAPtr = ptr >> 2;
	let pointBPtr = pointAPtr + 1;
	let pointNameStrPtr = Module.HEAP32[pointBPtr + 1];
	console.log('point.a', Module.HEAP32[pointAPtr]);
	console.log('point.b', Module.HEAP32[pointBPtr]);
	console.log('point.name', Module.AsciiToString(pointNameStrPtr));
}

function structToObj(ptr) {
	let pointAPtr = Module._getPoint() >> 2;
	let pointBPtr = pointAPtr + 1;
	let pointNameStrPtr = Module.HEAP32[pointBPtr + 1];

	return {
		a: Module.HEAP32[pointAPtr],
		b: Module.HEAP32[pointBPtr],
        name: Module.AsciiToString(pointNameStrPtr),
	}
}

function loadImgIntoMemEmscripten(img) {
	return new Promise(resolve => {
		fetch(img)
			.then(r => r.arrayBuffer())
			.then(buff => {
				const imgPtr = Module._malloc(buff.byteLength);
				const imgHeap = new Uint8Array(Module.HEAPU8.buffer, imgPtr, buff.byteLength);

				imgHeap.set(new Uint8Array(buff));

				resolve({ imgPtr });
			});
	});
}

function loadImgIntoMem(img, memory, alloc) {
	return new Promise(resolve => {
		fetch(img)
			.then(r => r.arrayBuffer())
			.then(buff => {
				const imgPtr = alloc(buff.byteLength);
				const imgHeap = new Uint8Array(memory.buffer, imgPtr, buff.byteLength);
				
				imgHeap.set(new Uint8Array(buff));

				resolve({ imgPtr, len: buff.byteLength });
			});
	});
}

function componentToHex(c) {
    var hex = c.toString(16);
    return hex.length == 1 ? "0" + hex : hex;
}

function rgbToHex(r, g, b) {
    return "#" + componentToHex(r) + componentToHex(g) + componentToHex(b);
}

function rgbArrToHex(arr) {
	document.querySelectorAll('.sample').forEach(i => document.body.removeChild(i));
	for (let i = 0; i < arr.length; i+=3) {
		const color = rgbToHex(arr[i], arr[i+1], arr[i+2]);

		const div = document.createElement('div');
		div.className = 'sample';
		div.style.backgroundColor = color;

		document.body.appendChild(div);
	}
}

function ptrToRGB(ptr) {
	return [...new Uint8Array(Module.instance.exports.memory.buffer, ptr, 3)];
}

function run(img, size) {
	return compile().then(m => {
		window.Module = m;
		window.Module.HEAP8 = new Int8Array(Module.instance.exports.memory.buffer);
		
		return loadImgIntoMem(img, m.instance.exports.memory, m.instance.exports.alloc).then(r => {
			return m.instance.exports.read_img(r.imgPtr, r.len, size);
		});
	})
	.then(ptr => {
		window.ptr = ptr;
		const HEAP16 = new Uint32Array(Module.instance.exports.memory.buffer);
		const colorsPtr = new Uint32Array(Module.instance.exports.memory.buffer, ptr, size * 3);
		const pixel1Ptr = HEAP16[colorsPtr / Uint32Array.BYTES_PER_ELEMENT];

		rgbArrToHex(ptrToRGB(pixel1Ptr));
	});
}

function compile(wasmFile = 'distil_wasm.gc.wasm') {
	return fetch(wasmFile)
        .then(r => r.arrayBuffer())
        .then(r => {
        	let module = new WebAssembly.Module(r);
        	let importObject = {}
        	for (let imp of WebAssembly.Module.imports(module)) {
        	    if (typeof importObject[imp.module] === "undefined")
        	        importObject[imp.module] = {};
        	    switch (imp.kind) {
        	    case "function": importObject[imp.module][imp.name] = () => {}; break;
        	    case "table": importObject[imp.module][imp.name] = new WebAssembly.Table({ initial: 256, maximum: 256, element: "anyfunc" }); break;
        	    case "memory": importObject[imp.module][imp.name] = new WebAssembly.Memory({ initial: 256 }); break;
        	    case "global": importObject[imp.module][imp.name] = 0; break;
        	    }
        	}
        	
        	importObject.env = Object.assign({}, importObject.env, {
        		log: (ptr, len) => console.log(ptrToStr(ptr, len)),
        		log_nr: (nr) => console.log(nr),
        	});

        	return WebAssembly.instantiate(r, importObject);
        });
}

function loadU8ArrToMem(arr) {
	var u8 = new Uint8Array(arr)
	var arrPtr = Module._malloc(u8.length)
	var arrHeap = new Uint8Array(Module.HEAPU8.buffer, arrPtr, u8.length)

	arrHeap.set(new Uint8Array(u8.buffer));

	return arrPtr;
}


function imgPtrToData(ptr, size) {
	const imgBuff = Module.HEAPU8.slice(ptr, size);
	return `data:image/jpg;base64,${btoa(String.fromCharCode(...imgBuff))}`;
}
