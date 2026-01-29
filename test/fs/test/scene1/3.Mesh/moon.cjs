require('../../../public/versionning/linkfile.cjs');

const moonRotation = new THREE.Object3D();
scene.add(moonRotation);
const moonMesh = new THREE.Mesh(
    new THREE.IcosahedronGeometry(0.27,12),
    new THREE.MeshPhongMaterial({
        map:loader.load(img.moonmap4k),
        bumpMap:loader.load(img.moonbump4k),
        bumpScale:4,
    })
) 

moonMesh.position.x = 8
moonMesh.castShadow = true;