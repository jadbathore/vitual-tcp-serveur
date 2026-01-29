require('../../../public/versionning/linkfile.cjs');

const renderer = new THREE.WebGLRenderer({antialias:true})
const scene = new THREE.Scene()
renderer.setSize(window.innerWidth,window.innerHeight);
document.body.appendChild(renderer.domElement)