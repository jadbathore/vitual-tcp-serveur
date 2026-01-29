require('../../../public/versionning/linkfile.cjs');

const earthGroup = new THREE.Group();

earthGroup.rotation.z = -23.4 * Math.PI / 180
scene.add(earthGroup);

const geo = new THREE.IcosahedronGeometry(1,12);
const earthMesh = new THREE.Mesh(
    geo,
    new THREE.MeshPhongMaterial(
        {
            bumpMap:loader.load(img.earthbump),
            specularMap:loader.load(img.earthspecular),
            map:loader.load(img.earthmap1k),
            bumpScale:7,
            shininess:13.0,
            specular: 0xFFFFFF,
            opacity:2,
        blending: THREE.AdditiveBlending,
        }
    )
)
earthGroup.add(earthMesh)
earthMesh.receiveShadow = true
const lightMesh = new THREE.Mesh(
    geo,
    new THREE.MeshBasicMaterial({
        lightMap:loader.load(img.earth_nightmap),
        transparent:true,
        opacity:0.2,
        blendAlpha:20,
        reflectivity:1,
        lightMapIntensity:20,
        blending: THREE.AdditiveBlending,
    })
)
earthGroup.add(lightMesh);

const couldsMat = new THREE.MeshBasicMaterial({
    map:loader.load(img.fair_clouds_8k),
    transparent:true,
    opacity:0.1,
    blending: THREE.AdditiveBlending,
})

const cloudMesh = new THREE.Mesh(
    geo,
    couldsMat
)
cloudMesh.scale.setScalar(1.01)
earthGroup.add(cloudMesh);

class test {
    constructor(hell){
        this.hell = hell
    }
    hello(){
        console.log(this.hell)
    }
}
