"""Recreate src/toys/bear.rs in Blender. Run through MCP execute_blender_code.

Set TOYBOX_ROOT in the execution namespace when using a different checkout.
Game (x, y, z) maps to Blender (x, z, y + .26); dimensions are preserved.
"""
from pathlib import Path
import bpy
from mathutils import Vector

root = Path(globals().get('TOYBOX_ROOT', r'D:\WebHatchery\RustGames\toybox'))
output = root / 'assets' / 'models'
output.mkdir(parents=True, exist_ok=True)
preview = root / 'docs' / 'verification' / 'blender_bear.png'
preview.parent.mkdir(parents=True, exist_ok=True)
scene = bpy.data.scenes.new('Toybox Bear Studio')
bpy.context.window.scene = scene
toy = bpy.data.collections.new('Bear - editable toy parts')
scene.collection.children.link(toy)

def material(name, rgb):
    mat = bpy.data.materials.new(name)
    mat.diffuse_color = (*rgb, 1)
    mat.use_nodes = True
    shader = mat.node_tree.nodes.get('Principled BSDF')
    shader.inputs['Base Color'].default_value = (*rgb, 1)
    shader.inputs['Roughness'].default_value = .78
    return mat

# Cherry palette, slot 0 / color index 0, including toy_color's warm/cool offsets.
base = (.772, .200, .172)
fur = material('Cherry plush', base)
head = material('Head highlight', tuple(min(1, c + .06) for c in base))
limb = material('Limb highlight', tuple(min(1, c + .05) for c in base))
tail = material('Tail highlight', tuple(min(1, c + .04) for c in base))
tan = material('Tan muzzle and pads', (.93, .80, .62))
ribbon = material('Cream bow', (.95, .92, .80))
ink = material('Face ink', (.035, .030, .026))
spark = material('Eye glints', (.92, .94, .90))

def part(name, position, size, mat, sphere=True):
    x, y, z = position
    location = (x, z, y + .26)
    if sphere:
        bpy.ops.mesh.primitive_uv_sphere_add(segments=8, ring_count=8,
                                           radius=size, location=location)
    else:
        bpy.ops.mesh.primitive_cube_add(size=1, location=location)
        bpy.context.object.scale = (size[0], size[2], size[1])
        bpy.ops.object.transform_apply(location=False, rotation=False, scale=True)
    obj = bpy.context.object
    obj.name = name
    obj.data.materials.append(mat)
    for collection in list(obj.users_collection):
        collection.objects.unlink(obj)
    toy.objects.link(obj)
    obj['source'] = 'src/toys/bear.rs / src/toys/primitives.rs'
    return obj

part('Body', (0, 0, 0), .26, fur)
part('Belly patch', (0, -.04, -.18), .13, tan)
part('Head', (0, .22, -.12), .17, head)
for side, x in [('Left', -.16), ('Right', .16)]:
    part(side + ' ear', (x, .36, -.12), .08, fur)
    part(side + ' inner ear', (x, .36, -.15), .045, tan)
    part(side + ' foot', (x, -.16, -.13), .065, limb)
    part(side + ' foot pad', (x, -.165, -.185), .035, tan)
part('Muzzle', (0, .18, -.26), .07, tan)
for side, x in [('Left', -.07), ('Right', .07)]:
    part(side + ' eye', (x, .22, -.28), (.052, .052, .016), ink, False)
    part(side + ' eye glint', (x - .010, .232, -.29), (.016, .016, .006), spark, False)
    part(side + ' cheek', (x * 1.25, .165, -.298), (.035, .018, .010), ink, False)
part('Nose', (0, .18, -.30), (.050, .034, .016), ink, False)
for side, x in [('Left', -.22), ('Right', .22)]:
    part(side + ' arm', (x, .04, -.10), .085, limb)
    part(side + ' paw pad', (x * 1.08, .02, -.165), .045, tan)
part('Tail', (0, -.06, .24), .07, tail)
part('Bow knot', (0, .10, -.24), .035, ribbon)
for side, x in [('Left', -.05), ('Right', .05)]:
    part(side + ' bow loop', (x, .10, -.235), (.055, .045, .022), ribbon, False)

# Export only the editable toy; studio geometry stays out of the game asset.
bpy.ops.object.select_all(action='DESELECT')
for obj in toy.objects:
    obj.select_set(True)
bpy.context.view_layer.objects.active = toy.objects['Body']
bpy.ops.export_scene.gltf(filepath=str(output / 'toybox_bear.glb'),
                         export_format='GLB', use_selection=True)

floor = material('Studio cream', (.30, .36, .35))
bpy.ops.mesh.primitive_plane_add(size=200, location=(0, 0, -.005))
bpy.context.object.name = 'Studio floor'
bpy.context.object.data.materials.append(floor)

def aim(obj, target):
    obj.rotation_euler = (Vector(target) - obj.location).to_track_quat('-Z', 'Y').to_euler()

for name, location, energy, size in [
    ('Key softbox', (-1.5, -2, 3), 180, 2),
    ('Fill softbox', (2, -1, 1.4), 75, 2),
    ('Rim softbox', (.5, 1.5, 2.2), 130, 1.5),
]:
    data = bpy.data.lights.new(name, 'AREA')
    data.energy, data.shape, data.size = energy, 'DISK', size
    obj = bpy.data.objects.new(name, data)
    scene.collection.objects.link(obj)
    obj.location = location
    aim(obj, (0, 0, .3))
bpy.ops.object.camera_add(location=(.95, -2.8, 1.0))
camera = bpy.context.object
camera.name = 'Bear portrait camera'
camera.data.type = 'ORTHO'
camera.data.ortho_scale = 1.08
aim(camera, (0, -.025, .34))
scene.camera = camera
scene.world = bpy.data.worlds.new('Bear studio world')
scene.world.use_nodes = True
scene.world.node_tree.nodes['Background'].inputs[0].default_value = (.30, .36, .40, 1)
scene.world.node_tree.nodes['Background'].inputs[1].default_value = .35
scene.render.engine = 'CYCLES'
scene.cycles.samples = 48
scene.cycles.use_denoising = True
scene.render.resolution_x = 1000
scene.render.resolution_y = 1000
scene.render.resolution_percentage = 100
scene.render.image_settings.file_format = 'PNG'
scene.render.filepath = str(preview)
scene.view_settings.view_transform = 'AgX'
bpy.ops.object.select_all(action='DESELECT')
for obj in toy.objects:
    obj.select_set(True)
bpy.context.view_layer.objects.active = toy.objects['Body']
for screen in bpy.data.screens:
    for area in screen.areas:
        if area.type == 'VIEW_3D':
            area.spaces.active.region_3d.view_perspective = 'CAMERA'
scene['source'] = 'Toybox Bear, src/toys/bear.rs; cherry palette slot 0'
bpy.ops.wm.save_as_mainfile(filepath=str(output / 'toybox_bear.blend'))
print('Saved bear with', len(toy.objects), 'editable parts')
