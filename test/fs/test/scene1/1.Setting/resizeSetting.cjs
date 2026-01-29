require('../../../public/versionning/linkfile.cjs');

window.addEventListener('resize',()=> {
    camera.aspect = window.innerWidth / window.innerHeight;
    camera.updateProjectionMatrix();
    renderer.setSize(window.innerWidth,window.innerHeight)
})
