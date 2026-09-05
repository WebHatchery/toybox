"""Reference-inspired plush bear. Execute in Blender or through Blender MCP.

Standalone authoring scene; leaves the original low-poly bear intact.
Fur is deterministic curve geometry, so no external texture files are needed.
"""
import math
import random
from pathlib import Path
import bpy
from mathutils import Vector

ROOT = Path(globals().get('TOYBOX_ROOT', r'D:\WebHatchery\RustGames\toybox'))
scene = bpy.data.scenes.new('Honey Bear - reference study')
bpy.context.window.scene = scene
collection = bpy.data.collections.new('Honey Bear')
scene.collection.children.link(collection)
rng = random.Random(20260906)

def material(name, color, roughness=.8, texture=False):
    mat = bpy.data.materials.new(name)
    mat.diffuse_color = (*color, 1)
    mat.use_nodes = True
    nodes, links = mat.node_tree.nodes, mat.node_tree.links
    bsdf = nodes.get('Principled BSDF')
    bsdf.inputs['Base Color'].default_value = (*color, 1)
    bsdf.inputs['Roughness'].default_value = roughness
    if texture:
        noise = nodes.new('ShaderNodeTexNoise')
        noise.inputs['Scale'].default_value = 160
        bump = nodes.new('ShaderNodeBump')
        bump.inputs['Strength'].default_value = .3
        bump.inputs['Distance'].default_value = .015
        links.new(noise.outputs['Fac'], bump.inputs['Height'])
        links.new(bump.outputs['Normal'], bsdf.inputs['Normal'])
    return mat

fur = material('Warm honey brown plush', (.24, .095, .030), texture=True)
hair = material('Golden brown fibers', (.33, .145, .050))
tan = material('Soft biscuit velour', (.57, .32, .145), texture=True)
brown = material('Chocolate embroidery', (.085, .027, .010), texture=True)
eye = material('Polished black glass eyes', (.009, .004, .002), .12)
bow = material('Burgundy and ivory gingham', (.4, .08, .05), texture=True)
n, l = bow.node_tree.nodes, bow.node_tree.links
coord = n.new('ShaderNodeTexCoord')
sep = n.new('ShaderNodeSeparateXYZ')
l.new(coord.outputs['Generated'], sep.inputs[0])
stripes = []
for axis in ['X', 'Z']:
    wave = n.new('ShaderNodeMath'); wave.operation = 'MULTIPLY'
    wave.inputs[1].default_value = 45
    l.new(sep.outputs[axis], wave.inputs[0])
    sine = n.new('ShaderNodeMath'); sine.operation = 'SINE'
    l.new(wave.outputs[0], sine.inputs[0])
    threshold = n.new('ShaderNodeMath'); threshold.operation = 'GREATER_THAN'
    l.new(sine.outputs[0], threshold.inputs[0])
    stripes.append(threshold)
add = n.new('ShaderNodeMath'); add.operation = 'ADD'
for i in range(2): l.new(stripes[i].outputs[0], add.inputs[i])
half = n.new('ShaderNodeMath'); half.operation = 'MULTIPLY'; half.inputs[1].default_value = .5
l.new(add.outputs[0], half.inputs[0])
ramp = n.new('ShaderNodeValToRGB')
ramp.color_ramp.elements[0].color = (.07, .008, .012, 1)
ramp.color_ramp.elements[1].color = (.78, .61, .39, 1)
ramp.color_ramp.elements.new(.5).color = (.31, .08, .065, 1)
l.new(half.outputs[0], ramp.inputs[0])
l.new(ramp.outputs[0], n.get('Principled BSDF').inputs['Base Color'])

def move_to_toy(obj):
    for col in list(obj.users_collection): col.objects.unlink(obj)
    collection.objects.link(obj)

def strand_object(name, strands, mat, thickness):
    data = bpy.data.curves.new(name, 'CURVE')
    data.dimensions = '3D'; data.resolution_u = 2
    data.bevel_depth = thickness; data.bevel_resolution = 0
    for points in strands:
        spline = data.splines.new('POLY')
        spline.points.add(len(points) - 1)
        for i, point in enumerate(points):
            spline.points[i].co = (*point, 1)
            spline.points[i].radius = 1 - .85 * i / max(1, len(points) - 1)
    obj = bpy.data.objects.new(name, data)
    collection.objects.link(obj); data.materials.append(mat)
    return obj

def ellipsoid(name, center, scale, mat, fibers=0, length=.038):
    bpy.ops.mesh.primitive_uv_sphere_add(segments=48, ring_count=32, location=center)
    obj = bpy.context.object; obj.name = name; obj.scale = scale
    obj.data.materials.append(mat); move_to_toy(obj)
    for poly in obj.data.polygons: poly.use_smooth = True
    if fibers:
        strands = []
        for _ in range(fibers):
            z = rng.uniform(-1, 1); phi = rng.uniform(0, math.tau)
            r = math.sqrt(1 - z*z)
            unit = Vector((r*math.cos(phi), r*math.sin(phi), z))
            p = Vector(center) + Vector(tuple(unit[i]*scale[i] for i in range(3)))
            normal = Vector(tuple(unit[i]/scale[i] for i in range(3))).normalized()
            tangent = normal.cross(Vector((.23, .41, 1))).normalized()
            cross = normal.cross(tangent)
            size = length * rng.uniform(.65, 1.4)
            curl = rng.uniform(0, math.tau)
            points = []
            for j in range(5):
                t = j/4
                wiggle = tangent*math.sin(t*4+curl) + cross*math.cos(t*5+curl)
                points.append(p + normal*(t*size) + wiggle*(size*.30*t))
            strands.append(points)
        strand_object(name + ' fur', strands, hair if mat == fur else mat, .0016)
    return obj

ellipsoid('Plump seated body', (0, .10, .88), (.65, .45, .77), fur, 16000)
ellipsoid('Oversized round head', (0, -.025, 1.91), (.70, .48, .64), fur, 18000)
for sign, side in [(-1, 'Left'), (1, 'Right')]:
    ellipsoid(side+' round ear', (sign*.58, .02, 2.37), (.255, .16, .28), fur, 2600)
    ellipsoid(side+' ear inset', (sign*.59, -.128, 2.38), (.15, .035, .175), brown, 650, .013)
    ellipsoid(side+' hugging arm', (sign*.60, -.03, .96), (.255, .29, .53), fur, 5000)
    ellipsoid(side+' oversized foot', (sign*.43, -.40, .36), (.355, .34, .36), fur, 4000)
    ellipsoid(side+' tan sole', (sign*.43, -.707, .36), (.287, .060, .284), tan, 1500, .010)
    ellipsoid(side+' central paw pad', (sign*.43, -.767, .255), (.136, .024, .117), brown, 400, .007)
    for j, dx in enumerate([-.175, -.065, .075, .18]):
        z = .48 if j in (1,2) else .405
        ellipsoid(side+' toe '+str(j+1), (sign*.43+dx, -.757, z), (.048, .024, .065), brown, 130, .006)
    ellipsoid(side+' eye socket', (sign*.245, -.441, 2.035), (.092, .040, .105), brown)
    ellipsoid(side+' glass eye', (sign*.245, -.478, 2.04), (.071, .042, .078), eye)
ellipsoid('Oval tan muzzle', (0, -.471, 1.83), (.345, .178, .25), tan, 2600, .010)
ellipsoid('Chocolate nose', (0, -.645, 1.94), (.123, .061, .087), brown, 500, .008)

# Embroidered smile follows the curved muzzle surface rather than a flat plane.
def muzzle_front(x, z):
    return -.471 - .178*math.sqrt(max(.05, 1-(x/.345)**2-((z-1.83)/.25)**2)) - .012
smile = []
for i in range(41):
    x = -.205 + .41*i/40; z = 1.745 + .078*(x/.205)**2
    smile.append((x, muzzle_front(x,z), z))
stem = [(0, muzzle_front(0,z), z) for z in [1.91, 1.87, 1.83, 1.79, 1.745]]
strand_object('Stitched happy smile', [smile, stem], brown, .007)

def cloth(name, vertices, faces):
    mesh = bpy.data.meshes.new(name); mesh.from_pydata(vertices, [], faces); mesh.update()
    obj = bpy.data.objects.new(name, mesh); collection.objects.link(obj)
    mesh.materials.append(bow)
    for face in mesh.polygons: face.use_smooth = True
    sub = obj.modifiers.new('Soft fabric folds', 'SUBSURF'); sub.levels = 2
    solid = obj.modifiers.new('Fabric thickness', 'SOLIDIFY'); solid.thickness = .008
    return obj

for sign, side in [(-1,'Left'), (1,'Right')]:
    verts, faces = [], []
    for i in range(13):
        t = i/12
        for j in range(9):
            v = j/8*2-1
            verts.append((sign*(.055+.32*t), -.45-.065*math.sin(math.pi*t)-.030*math.cos(v*math.pi*2),
                          1.40 + v*(.055+.14*math.sin(t*math.pi*.70)) + .03*t))
    for i in range(12):
        for j in range(8):
            a=i*9+j; faces.append((a,a+1,a+10,a+9))
    cloth(side+' gathered bow loop', verts, faces)
    verts, faces = [], []
    for i in range(10):
        t=i/9
        for j in range(5):
            v=j/4*2-1
            verts.append((sign*(.075+.15*t)+v*.090, -.46-.12*t+.022*math.cos(v*4),
                          1.35-.39*t+.035*v*t))
    for i in range(9):
        for j in range(4):
            a=i*5+j; faces.append((a,a+1,a+6,a+5))
    cloth(side+' hanging ribbon', verts, faces)
ellipsoid('Gathered bow knot', (0, -.49, 1.39), (.077, .070, .096), bow)

# Warm wooden display shelf, with a quiet studio background.
wood = material('Honey oak shelf', (.25, .115, .044), texture=True)
bpy.ops.mesh.primitive_cube_add(size=1, location=(0, 0, -.085))
shelf=bpy.context.object; shelf.name='Wooden display shelf'; shelf.scale=(200,200,.12)
shelf.data.materials.append(wood)
scene.world=bpy.data.worlds.new('Warm shop ambience'); scene.world.use_nodes=True
scene.world.node_tree.nodes['Background'].inputs[0].default_value=(.30,.22,.15,1)
scene.world.node_tree.nodes['Background'].inputs[1].default_value=.35

def aim(obj, target):
    obj.rotation_euler=(Vector(target)-obj.location).to_track_quat('-Z','Y').to_euler()
for name, pos, power, size in [('Warm key',(-3,-4,5),650,4),('Soft fill',(3,-2,3),260,3),('Fur rim',(0,2,4),750,3)]:
    data=bpy.data.lights.new(name,'AREA'); data.energy=power; data.shape='DISK'; data.size=size
    obj=bpy.data.objects.new(name,data); scene.collection.objects.link(obj); obj.location=pos
    aim(obj,(0,0,1.4))
bpy.ops.object.camera_add(location=(.35,-6.8,2.9))
camera=bpy.context.object; camera.name='Plush portrait camera'; camera.data.type='ORTHO'
camera.data.ortho_scale=3.10; aim(camera,(0,-.04,1.30)); scene.camera=camera
scene.render.engine='CYCLES'; scene.cycles.samples=48; scene.cycles.use_denoising=True
scene.render.resolution_x=1100; scene.render.resolution_y=1100; scene.render.resolution_percentage=100
scene.render.image_settings.file_format='PNG'
scene.render.filepath=str(ROOT/'docs/verification/blender_plush_bear.png')
scene.view_settings.view_transform='AgX'
for screen in bpy.data.screens:
    for area in screen.areas:
        if area.type=='VIEW_3D': area.spaces.active.region_3d.view_perspective='CAMERA'
bpy.ops.object.select_all(action='DESELECT')
scene['reference']='User supplied brown plush teddy photograph; shape, fur, paws and gingham bow study'
scene['fur_note']='Deterministic geometric fiber curves; authoring/render asset, not a game-ready mesh'
bpy.ops.wm.save_as_mainfile(filepath=str(ROOT/'assets/models/toybox_plush_bear.blend'))
print('Saved reference-inspired plush bear:',len(collection.objects),'objects')
