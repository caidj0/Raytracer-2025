import bpy
import math
import mathutils
import json
from bpy_extras.io_utils import ExportHelper
from bpy.props import StringProperty
from bpy.types import Operator

def blender_to_engine_coords(v):
    return mathutils.Vector((v.x, v.z, -v.y))  # Z-up to Y-up

def get_camera_parameters():
    scene = bpy.context.scene
    camera = scene.camera
    cam_data = camera.data

    image_width = scene.render.resolution_x
    image_height = scene.render.resolution_y
    aspect_ratio = image_width / image_height

    if cam_data.type != 'PERSP':
        raise Exception("Only perspective cameras are supported.")

    sensor_height = cam_data.sensor_height if cam_data.sensor_fit != 'VERTICAL' else cam_data.sensor_width
    vertical_fov = math.degrees(cam_data.angle) / 2

    cam_world_matrix = camera.matrix_world
    look_from = cam_world_matrix.to_translation()
    forward = cam_world_matrix.to_3x3() @ mathutils.Vector((0.0, 0.0, -1.0))
    vec_up = cam_world_matrix.to_3x3() @ mathutils.Vector((0.0, 1.0, 0.0))
    look_at = look_from + forward

    look_from_conv = blender_to_engine_coords(look_from)
    look_at_conv = blender_to_engine_coords(look_at)
    vec_up_conv = blender_to_engine_coords(vec_up).normalized()

    if cam_data.dof.use_dof:
        focus_distance = cam_data.dof.focus_distance
        aperture_radius = cam_data.dof.aperture_fstop
        defocus_angle = math.degrees(math.atan(1.0 / (2.0 * aperture_radius))) if aperture_radius > 0 else 0.0
    else:
        focus_distance = (look_at - look_from).length
        defocus_angle = 0.0

    camera_params = {
        "aspect_ratio": aspect_ratio,
        "image_width": image_width,
        "vertical_fov_in_degrees": vertical_fov,
        "look_from": list(look_from_conv),
        "look_at": list(look_at_conv),
        "vec_up": list(vec_up_conv),
        "defocus_angle_in_degrees": defocus_angle,
        "focus_distance": focus_distance
    }

    return camera_params

class ExportCameraParamsOperator(Operator, ExportHelper):
    bl_idname = "export_scene.camera_params"
    bl_label = "Export Camera Parameters"
    filename_ext = ".json"
    filter_glob: StringProperty(default="*.json", options={'HIDDEN'})

    def execute(self, context):
        try:
            params = get_camera_parameters()
            with open(self.filepath, 'w') as f:
                json.dump(params, f, indent=4)
            self.report({'INFO'}, f"Camera parameters exported to {self.filepath}")
            return {'FINISHED'}
        except Exception as e:
            self.report({'ERROR'}, str(e))
            return {'CANCELLED'}

# 注册操作符后可从搜索栏或脚本运行
def menu_func_export(self, context):
    self.layout.operator(ExportCameraParamsOperator.bl_idname, text="Export Camera Parameters (.json)")

def register():
    bpy.utils.register_class(ExportCameraParamsOperator)
    bpy.types.TOPBAR_MT_file_export.append(menu_func_export)

def unregister():
    bpy.utils.unregister_class(ExportCameraParamsOperator)
    bpy.types.TOPBAR_MT_file_export.remove(menu_func_export)

if __name__ == "__main__":
    register()
    # 立即弹出导出窗口
    bpy.ops.export_scene.camera_params('INVOKE_DEFAULT')
