# 渲染
1. 实现了 book3 的 PDF 方法，并撰写了一份[报告](<PDF 方法在光线追踪的应用.pdf>)；
2. [实现](../src/shapes/obj.rs#L20-L91)了纹理、法线、Alpha 映射，[实现](../src/texture.rs#L121)了双线性插值；
3. 高级材质：[实现](../src/material/disney.rs)了 Disney BSDF；
4. [实现](../src/shapes/environment.rs)了环境照明（没有 PDF）；
5. 更多视觉效果：[实现](../src/material/portal.rs)了光线传送材质

# 几何 
[实现](../src/shapes/obj.rs)了 obj 文件的加载；在绕 Y 轴旋转的基础上[实现](../src/shapes.rs#L23-L130)了物体的变换

# 优化
通过 rayon 库[实现](../src/camera.rs#L180)了多线程