require('../../../public/versionning/linkfile.cjs');

const sunLight = new THREE.DirectionalLight(0xFFFFFF)
scene.add(sunLight);
sunLight.position.set(10,0,0);
sunLight.castShadow = true;
sunLight.shadow.mapSize.width = 1024;
sunLight.shadow.mapSize.height = 1024;