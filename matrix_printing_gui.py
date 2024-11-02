import tkinter as tk
from tkinter import ttk, filedialog, messagebox
from PIL import Image, ImageDraw, ImageFont, ImageTk
import math
import json
import os
import shutil
from datetime import datetime

class MatrixPrintingGUI:
    def __init__(self, root):
        self.root = root
        self.root.title("Matrix Printing Tool")
        
        # 创建三栏布局
        self.left_frame = ttk.Frame(root, padding="10")
        self.middle_frame = ttk.Frame(root, padding="10")
        self.right_frame = ttk.Frame(root, padding="10")
        self.left_frame.pack(side=tk.LEFT, fill=tk.BOTH)
        self.middle_frame.pack(side=tk.LEFT, fill=tk.BOTH)
        self.right_frame.pack(side=tk.LEFT, fill=tk.BOTH)
        
        # 图片相关变量
        self.image_path = None
        self.image = None
        self.preview_image = None
        self.font_path = "LXGWWenKai-Regular.ttf"  # 默认字体
        
        # 参数变量
        self.start_x = tk.StringVar()
        self.start_y = tk.StringVar()
        self.cell_width = tk.StringVar()
        self.cell_height = tk.StringVar()
        self.font_size = tk.StringVar()
        self.offset_x = tk.StringVar(value="1")
        self.offset_y = tk.StringVar(value="-1")
        self.grid_columns = tk.StringVar()
        self.grid_rows = tk.StringVar()
        self.grid_line_thickness = tk.StringVar(value="1")  # 添加线条粗细参数
        self.first_line_indent = tk.BooleanVar(value=True)  # 首段缩进控制
        self.first_line_newline = tk.BooleanVar(value=False)  # 首行换行控制
        
        # 创建必要的文件夹并清理旧文件
        self.folders = {
            'fonts': 'fonts',
            'uploads': 'uploads',
            'output': 'output',
            'config': 'config'  # 添加 config 文件夹
        }
        self.create_folders()
        self.cleanup_uploads()  # 在启动时清理旧文件
        
        # 字体相关变量
        self.fonts_list = []
        self.selected_font = tk.StringVar()
        self.load_available_fonts()
        
        # 添加输出尺寸选择
        self.output_sizes = {
            "原始尺寸": "original",
            "A4 (2480x3508)": (2480, 3508),
            "A3 (3508x4961)": (3508, 4961),
            "4K (3840x2160)": (3840, 2160),
            "高清 (1920x1080)": (1920, 1080)
        }
        self.selected_size = tk.StringVar(value="原始尺寸")
        
        self.setup_ui()
        self.load_default_settings()
    
    def setup_ui(self):
        # 左侧面板：图片和参数设置
        self.setup_left_panel()
        
        # 中间面板：文本输入
        self.setup_middle_panel()
        
        # 右侧面板：预览
        self.setup_right_panel()
    
    def setup_left_panel(self):
        # 1. 输出尺寸选择
        size_frame = ttk.LabelFrame(self.left_frame, text="输出尺寸", padding="5")
        size_frame.pack(fill=tk.X, pady=5)
        
        size_combo = ttk.Combobox(size_frame, 
                                 textvariable=self.selected_size,
                                 values=list(self.output_sizes.keys()),
                                 state="readonly")
        size_combo.pack(fill=tk.X, pady=5)
        size_combo.bind('<<ComboboxSelected>>', self.on_size_changed)
        
        # 2. 网格参数（移到上传按钮之前）
        grid_frame = ttk.LabelFrame(self.left_frame, text="网格参数", padding="5")
        grid_frame.pack(fill=tk.X, pady=5)
        
        ttk.Label(grid_frame, text="每行格子数:").pack()
        ttk.Entry(grid_frame, textvariable=self.grid_columns).pack(fill=tk.X, pady=2)
        
        ttk.Label(grid_frame, text="格子行数:").pack()
        ttk.Entry(grid_frame, textvariable=self.grid_rows).pack(fill=tk.X, pady=2)
        
        # 3. 上传按钮
        ttk.Button(self.left_frame, text="上传格子图片", command=self.load_image).pack(pady=5)
        ttk.Button(self.left_frame, text="上传字体文件", command=self.load_font).pack(pady=5)
        
        # 4. 详细参数调整
        params_frame = ttk.LabelFrame(self.left_frame, text="参数调整", padding="5")
        params_frame.pack(fill=tk.X, pady=5)
        
        # 添加字体选择
        font_frame = ttk.Frame(params_frame)
        font_frame.pack(fill=tk.X)
        ttk.Label(font_frame, text="选择字体:").pack(side=tk.LEFT)
        font_combo = ttk.Combobox(font_frame, 
                                 textvariable=self.selected_font,
                                 values=self.fonts_list,
                                 state="readonly")
        font_combo.pack(side=tk.RIGHT, fill=tk.X, expand=True)
        
        # 保存下拉框的引用名称
        self.font_combo_name = str(font_combo)
        
        params = [
            ("起始X坐标:", self.start_x),
            ("起始Y坐标:", self.start_y),
            ("格子宽度:", self.cell_width),
            ("格子高度:", self.cell_height),
            ("线条粗细:", self.grid_line_thickness),  # 移动到字体大小之前
            ("字体大小:", self.font_size),
            ("X轴偏移:", self.offset_x),
            ("Y轴偏移:", self.offset_y),
        ]
        
        for label, var in params:
            frame = ttk.Frame(params_frame)
            frame.pack(fill=tk.X)
            ttk.Label(frame, text=label).pack(side=tk.LEFT)
            entry = ttk.Entry(frame, textvariable=var)
            entry.pack(side=tk.RIGHT)
            var.trace_add("write", self.update_preview)
        
        # 参数预设
        preset_frame = ttk.LabelFrame(self.left_frame, text="参数预设", padding="5")
        preset_frame.pack(fill=tk.X, pady=5)
        ttk.Button(preset_frame, text="保存当前参数", command=self.save_settings).pack(pady=2)
        ttk.Button(preset_frame, text="加载保存的参数", command=self.load_settings).pack(pady=2)
        
        # 为所有参数添加跟踪
        params = [self.start_x, self.start_y, self.cell_width, self.cell_height,
                  self.font_size, self.offset_x, self.offset_y, 
                  self.grid_columns, self.grid_rows]
        
        for var in params:
            var.trace_add("write", self.update_preview)
        
        # 为字体选择添加跟踪
        self.selected_font.trace_add("write", self.update_preview)
        
        # 为缩进和换行选项添加跟踪
        self.first_line_indent.trace_add("write", self.update_preview)
        self.first_line_newline.trace_add("write", self.update_preview)
    
    def setup_middle_panel(self):
        # 文本输入区域
        text_frame = ttk.LabelFrame(self.middle_frame, text="文本输入", padding="5")
        text_frame.pack(fill=tk.BOTH, expand=True)
        
        self.text_input = tk.Text(text_frame, width=40, height=20)
        self.text_input.pack(fill=tk.BOTH, expand=True)
        
        # 添加文本变化监听
        self.text_input.bind('<<Modified>>', self.on_text_changed)
        
        # 文本格式选项
        format_frame = ttk.LabelFrame(self.middle_frame, text="文本格式", padding="5")
        format_frame.pack(fill=tk.X, pady=5)
        
        ttk.Checkbutton(format_frame, text="首行缩进", variable=self.first_line_indent).pack()
        ttk.Checkbutton(format_frame, text="首行换行", variable=self.first_line_newline).pack()
        
        # 生成按钮
        ttk.Button(self.middle_frame, text="生成图片", command=self.generate_image).pack(pady=10)
    
    def setup_right_panel(self):
        self.right_frame.pack(side=tk.LEFT, fill=tk.BOTH, expand=True)  # 添加 expand=True
        
        preview_frame = ttk.LabelFrame(self.right_frame, text="预览", padding="5")
        preview_frame.pack(fill=tk.BOTH, expand=True)
        
        # 创建Canvas并设置为可扩展
        self.preview_canvas = tk.Canvas(preview_frame)
        self.preview_canvas.pack(fill=tk.BOTH, expand=True)
        
        # 绑定窗口大小变化事件
        self.root.bind('<Configure>', self.on_window_resize)
    
    def load_image(self):
        """上传并处理图片"""
        # 先检查网格参数
        if not self.grid_columns.get() or not self.grid_rows.get():
            messagebox.showerror("错误", "请先输入网格参数（每行格子数和格子行数）")
            return
            
        try:
            columns = int(self.grid_columns.get())
            rows = int(self.grid_rows.get())
            if columns <= 0 or rows <= 0:
                messagebox.showerror("错误", "网格参数必须为正整数")
                return
        except ValueError:
            messagebox.showerror("错误", "网格参数必须为有效的数字")
            return
        
        # 选择文件
        file_path = filedialog.askopenfilename(
            filetypes=[("Image files", "*.png *.jpg *.jpeg *.gif *.bmp")]
        )
        
        if file_path:
            # 生成唯一文件名
            timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
            filename = f"upload_{timestamp}{os.path.splitext(file_path)[1]}"
            upload_path = os.path.join(self.folders['uploads'], filename)
            
            # 复制文件到 uploads 文件夹
            shutil.copy2(file_path, upload_path)
            self.image_path = upload_path
            
            # 保存原始图片并进行尺寸调整
            self.original_image = Image.open(upload_path)
            self.resize_image()
            self.calculate_params()  # 自动计算参数
            self.update_preview()
    
    def load_font(self):
        """上传并制字体到 fonts 文件夹"""
        font_path = filedialog.askopenfilename(
            filetypes=[("Font files", "*.ttf *.otf")]
        )
        if font_path:
            # 生成唯一文件名
            filename = os.path.basename(font_path)
            dest_path = os.path.join(self.folders['fonts'], filename)
            
            try:
                # 检查文件是否已存在
                if os.path.exists(dest_path):
                    if not messagebox.askyesno("文件已存在", 
                        f"字体文件 {filename} 已存在。是否替换？"):
                        return
                
                # 复制文件到 fonts 文件夹
                shutil.copy2(font_path, dest_path)
                
                # 重新加载字体列表
                self.load_available_fonts()
                
                # 更新下拉框的值
                font_combo = self.root.nametowidget(self.font_combo_name)  # 获取下拉框组件
                font_combo['values'] = self.fonts_list  # 更新可选值
                
                # 选择新上传的字体
                self.selected_font.set(filename)
                
                messagebox.showinfo("成功", "字体文件已添加")
            except Exception as e:
                messagebox.showerror("错误", f"添加字体失败: {str(e)}")
    
    def save_settings(self):
        settings = {
            'start_x': format(float(self.start_x.get()), '.1f'),
            'start_y': format(float(self.start_y.get()), '.1f'),
            'cell_width': format(float(self.cell_width.get()), '.1f'),
            'cell_height': format(float(self.cell_height.get()), '.1f'),
            'font_size': self.font_size.get(),
            'offset_x': self.offset_x.get(),
            'offset_y': self.offset_y.get(),
            'grid_columns': self.grid_columns.get(),
            'grid_rows': self.grid_rows.get(),
            'grid_line_thickness': format(float(self.grid_line_thickness.get()), '.1f'),
        }
        
        # 弹出对话框让用户输入配置文件名
        filename = filedialog.asksaveasfilename(
            initialdir=self.folders['config'],
            title="保存参数配置",
            defaultextension=".json",
            filetypes=[("JSON files", "*.json")]
        )
        
        if filename:  # 如果用户没有取消
            try:
                with open(filename, 'w', encoding='utf-8') as f:
                    json.dump(settings, f, ensure_ascii=False, indent=4)
                messagebox.showinfo("成功", "参数配置已保存")
            except Exception as e:
                messagebox.showerror("错误", f"保存配置失败: {str(e)}")
    
    def load_settings(self):
        try:
            # 弹出对话框让用户选择配置件
            filename = filedialog.askopenfilename(
                initialdir=self.folders['config'],
                title="加载参数配置",
                filetypes=[("JSON files", "*.json")]
            )
            
            if filename:  # 如果用户没有取消
                with open(filename, 'r', encoding='utf-8') as f:
                    settings = json.load(f)
                    self.start_x.set(settings.get('start_x', '0.0'))
                    self.start_y.set(settings.get('start_y', '0.0'))
                    self.cell_width.set(settings.get('cell_width', '0.0'))
                    self.cell_height.set(settings.get('cell_height', '0.0'))
                    self.font_size.set(settings.get('font_size', ''))
                    self.offset_x.set(settings.get('offset_x', '1'))
                    self.offset_y.set(settings.get('offset_y', '-1'))
                    self.grid_columns.set(settings.get('grid_columns', ''))
                    self.grid_rows.set(settings.get('grid_rows', ''))
                    self.grid_line_thickness.set(settings.get('grid_line_thickness', '1.0'))
                    messagebox.showinfo("成功", "参数配置已加载")
        except FileNotFoundError:
            messagebox.showerror("错误", "未找到配置文件")
        except Exception as e:
            messagebox.showerror("错误", f"加载配置失败: {str(e)}")
    
    def generate_image(self):
        """修改生成图片的方法，移除尺寸选择对话框"""
        if not self.image:
            messagebox.showerror("错误", "请先上传图片")
            return
            
        try:
            # 创建新图像并绘制文本
            result_image = self.image.copy()
            self.draw_text_on_image(result_image)
            
            # 保存结果
            timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
            default_filename = f"output_{timestamp}.png"
            save_path = filedialog.asksaveasfilename(
                initialdir=self.folders['output'],
                initialfile=default_filename,
                defaultextension=".png",
                filetypes=[("PNG files", "*.png")]
            )
            if save_path:
                result_image.save(save_path, dpi=(300, 300))
                result_image.show()
                messagebox.showinfo("成功", "图片已生成并保存")
                
        except Exception as e:
            messagebox.showerror("错误", f"生成失败: {str(e)}")
    
    def draw_text_on_image(self, result_image):
        """在图片上绘制文本"""
        draw = ImageDraw.Draw(result_image)
        
        # 获取参数
        try:
            start_x = float(self.start_x.get())
            start_y = float(self.start_y.get())
            cell_width = float(self.cell_width.get())
            cell_height = float(self.cell_height.get())
            grid_width = int(self.grid_columns.get())
            font_size = int(self.font_size.get())
            offset_x = int(self.offset_x.get())
            offset_y = int(self.offset_y.get())
            line_thickness = float(self.grid_line_thickness.get())  # 添加到参数列表
            
            # 计算实际的格子尺寸（包含线条厚度）
            actual_cell_width = cell_width + line_thickness
            actual_cell_height = cell_height + line_thickness
            
            # 创建字体对象
            try:
                font_path = os.path.join(self.folders['fonts'], self.selected_font.get())
                font = ImageFont.truetype(font_path, font_size)
            except Exception:
                raise Exception("字体加载失败")
            
            # 获取文本并分段
            text = self.text_input.get("1.0", tk.END).strip()
            paragraphs = text.split('\n\n')  # 通过双换行符分割段落
            
            # 绘制文本
            current_x, current_y = start_x, start_y
            line_char_count = 0
            
            for i, para in enumerate(paragraphs):
                # 处理首行换行
                if i == 0 and self.first_line_newline.get():
                    current_x = start_x
                    current_y += actual_cell_height
                    line_char_count = 0
                
                # 处理段落缩进
                if i == 0 and self.first_line_indent.get():
                    current_x += actual_cell_width * 2
                    line_char_count = 2
                elif i > 0:
                    if line_char_count > 0:
                        current_x = start_x
                        current_y += actual_cell_height
                        line_char_count = 0
                    current_x += actual_cell_width * 2
                    line_char_count = 2
                
                # 绘制段落中的每个字符
                for char in para.strip():
                    if line_char_count >= grid_width:
                        current_x = start_x
                        current_y += actual_cell_height
                        line_char_count = 0
                    
                    # 计算字符位置（居中对齐）
                    bbox = font.getbbox(char)
                    char_width = bbox[2] - bbox[0]
                    char_height = bbox[3] - bbox[1]
                    
                    char_x = current_x + (actual_cell_width - char_width) / 2 + offset_x
                    char_y = current_y + (actual_cell_height - char_height) / 2 + offset_y
                    
                    draw.text((char_x, char_y), char, fill="black", font=font)
                    
                    current_x += actual_cell_width
                    line_char_count += 1
            
            return result_image
        
        except ValueError:
            raise ValueError("请确保所有参数都是有效的数值")
    
    def create_folders(self):
        """创建必要的文件夹"""
        for folder in self.folders.values():
            os.makedirs(folder, exist_ok=True)

    def load_available_fonts(self):
        """加载 fonts 文件夹中的所有字体文件"""
        self.fonts_list = [f for f in os.listdir(self.folders['fonts']) 
                          if f.lower().endswith(('.ttf', '.otf'))]
        if self.fonts_list:
            self.selected_font.set(self.fonts_list[0])

    def cleanup_uploads(self, days=7):
        """清理超过指定天数的上传文件"""
        try:
            current_time = datetime.now()
            for filename in os.listdir(self.folders['uploads']):
                file_path = os.path.join(self.folders['uploads'], filename)
                file_time = datetime.fromtimestamp(os.path.getctime(file_path))
                if (current_time - file_time).days > days:
                    try:
                        os.remove(file_path)
                    except Exception as e:
                        print(f"清理文件失败: {filename}, 错误: {str(e)}")
        except Exception as e:
            print(f"清理上传文件夹失败: {str(e)}")

    def calculate_params(self):
        """Calculate suggested parameters based on the uploaded image"""
        if not self.image:
            messagebox.showerror("错误", "请先上传图片")
            return
            
        # Get image dimensions
        img_width, img_height = self.image.size
        
        # Get grid dimensions from user input
        try:
            columns = int(self.grid_columns.get())
            rows = int(self.grid_rows.get())
        except ValueError:
            messagebox.showerror("错误", "请先输入有效行数和列数")
            return
        
        # Calculate cell dimensions
        cell_width = img_width // columns
        cell_height = img_height // rows
        
        # Set calculated parameters
        self.start_x.set("0")
        self.start_y.set("0")
        self.cell_width.set(str(cell_width))
        self.cell_height.set(str(cell_height))
        self.font_size.set(str(min(cell_width, cell_height) - 4))  # Slightly smaller than cell size
        
        messagebox.showinfo("成功", "参数已更新")
        self.update_preview()

    def on_text_changed(self, event):
        """处理文本变化事件"""
        self.text_input.edit_modified(False)  # 重置modified标志
        self.update_preview()

    def on_window_resize(self, event):
        """处理窗口大小变化"""
        if hasattr(self, 'last_preview_image'):
            self.update_preview()

    def update_preview(self, *args):
        """更新预览图像，包含网格和文本"""
        if not self.image:
            return
            
        try:
            # 创建原图副本
            preview = self.image.copy()
            draw = ImageDraw.Draw(preview)
            
            # 获取参数
            try:
                start_x = float(self.start_x.get()) if self.start_x.get() else 0.0
                start_y = float(self.start_y.get()) if self.start_y.get() else 0.0
                cell_width = float(self.cell_width.get()) if self.cell_width.get() else 50.0
                cell_height = float(self.cell_height.get()) if self.cell_height.get() else 50.0
                columns = int(self.grid_columns.get()) if self.grid_columns.get() else 10
                rows = int(self.grid_rows.get()) if self.grid_rows.get() else 10
                font_size = int(self.font_size.get()) if self.font_size.get() else 30
                offset_x = int(self.offset_x.get()) if self.offset_x.get() else 0
                offset_y = int(self.offset_y.get()) if self.offset_y.get() else 0
                line_thickness = float(self.grid_line_thickness.get()) if self.grid_line_thickness.get() else 1.0
                
                # 计算实际的格子尺寸
                actual_cell_width = cell_width + line_thickness
                actual_cell_height = cell_height + line_thickness
                
                # 绘制网格线
                for i in range(columns + 1):
                    x = start_x + i * actual_cell_width
                    # 将浮点数坐标转换为整数
                    x = round(x)
                    y1 = round(start_y)
                    y2 = round(start_y + rows * actual_cell_height)
                    draw.line([(x, y1), (x, y2)], 
                             fill="red", width=round(line_thickness))
                
                for i in range(rows + 1):
                    y = start_y + i * actual_cell_height
                    # 将浮点数坐标转换为整数
                    y = round(y)
                    x1 = round(start_x)
                    x2 = round(start_x + columns * actual_cell_width)
                    draw.line([(x1, y), (x2, y)], 
                             fill="red", width=round(line_thickness))
                
                # 如果有文本且有选择字体，绘制文本
                text = self.text_input.get("1.0", tk.END).strip()
                if text and self.selected_font.get():
                    try:
                        font_path = os.path.join(self.folders['fonts'], self.selected_font.get())
                        font = ImageFont.truetype(font_path, font_size)
                        
                        current_x, current_y = start_x, start_y
                        line_char_count = 0
                        
                        paragraphs = text.split('\n\n')
                        
                        for i, para in enumerate(paragraphs):
                            # 处理首行换行
                            if i == 0 and self.first_line_newline.get():
                                current_x = start_x
                                current_y += actual_cell_height
                                line_char_count = 0
                            
                            # 处理首行缩进
                            if i == 0 and self.first_line_indent.get():
                                current_x += actual_cell_width * 2
                                line_char_count = 2
                            elif i > 0:
                                if line_char_count > 0:
                                    current_x = start_x
                                    current_y += actual_cell_height
                                    line_char_count = 0
                                current_x += actual_cell_width * 2
                                line_char_count = 2
                            
                            for char in para.strip():
                                if line_char_count >= columns:
                                    current_x = start_x
                                    current_y += actual_cell_height
                                    line_char_count = 0
                                
                                bbox = font.getbbox(char)
                                char_width = bbox[2] - bbox[0]
                                char_height = bbox[3] - bbox[1]
                                
                                # 使用实际格子尺寸计算字符位置
                                char_x = current_x + (actual_cell_width - char_width) / 2 + offset_x
                                char_y = current_y + (actual_cell_height - char_height) / 2 + offset_y
                                
                                draw.text((char_x, char_y), char, fill="blue", font=font)
                                
                                current_x += actual_cell_width
                                line_char_count += 1
                            
                    except Exception as e:
                        print(f"预览文本绘制失败: {str(e)}")
                
                # 保存原始预览图像
                self.last_preview_image = preview
                
                # 获取Canvas当大小
                canvas_width = self.preview_canvas.winfo_width()
                canvas_height = self.preview_canvas.winfo_height()
                
                if canvas_width > 1 and canvas_height > 1:
                    # 计算缩放比例
                    width_ratio = canvas_width / preview.width
                    height_ratio = canvas_height / preview.height
                    ratio = min(width_ratio, height_ratio)
                    
                    # 计算新尺寸
                    new_width = int(preview.width * ratio)
                    new_height = int(preview.height * ratio)
                    
                    # 调整图像大小
                    preview_resized = preview.resize((new_width, new_height), Image.LANCZOS)
                    
                    # 转换并显示
                    self.preview_image = ImageTk.PhotoImage(preview_resized)
                    
                    # 清除Canvas并居中显示图像
                    self.preview_canvas.delete("all")
                    x = (canvas_width - new_width) // 2
                    y = (canvas_height - new_height) // 2
                    self.preview_canvas.create_image(x, y, anchor="nw", image=self.preview_image)
                
            except (ValueError, tk.TclError) as e:
                print(f"参数错误: {str(e)}")
                # 如果参数无效，至少显示原图
                self.show_original_preview(preview)
                
        except Exception as e:
            print(f"预览更新失败: {str(e)}")

    def show_original_preview(self, preview):
        """显示原始图片预览（当参数无效时使用）"""
        canvas_width = self.preview_canvas.winfo_width()
        canvas_height = self.preview_canvas.winfo_height()
        
        if canvas_width > 1 and canvas_height > 1:
            width_ratio = canvas_width / preview.width
            height_ratio = canvas_height / preview.height
            ratio = min(width_ratio, height_ratio)
            
            new_width = int(preview.width * ratio)
            new_height = int(preview.height * ratio)
            
            preview_resized = preview.resize((new_width, new_height), Image.LANCZOS)
            self.preview_image = ImageTk.PhotoImage(preview_resized)
            
            self.preview_canvas.delete("all")
            x = (canvas_width - new_width) // 2
            y = (canvas_height - new_height) // 2
            self.preview_canvas.create_image(x, y, anchor="nw", image=self.preview_image)

    def load_default_settings(self):
        """加载默认配置文件，如果不存在则使用默认值"""
        default_config = os.path.join(self.folders['config'], 'default.json')
        try:
            if os.path.exists(default_config):
                with open(default_config, 'r', encoding='utf-8') as f:
                    settings = json.load(f)
                    self.start_x.set(settings.get('start_x', '0.0'))
                    self.start_y.set(settings.get('start_y', '0.0'))
                    self.cell_width.set(settings.get('cell_width', '0.0'))
                    self.cell_height.set(settings.get('cell_height', '0.0'))
                    self.font_size.set(settings.get('font_size', ''))
                    self.offset_x.set(settings.get('offset_x', '1'))
                    self.offset_y.set(settings.get('offset_y', '-1'))
                    self.grid_columns.set(settings.get('grid_columns', ''))
                    self.grid_rows.set(settings.get('grid_rows', ''))
                    self.grid_line_thickness.set(settings.get('grid_line_thickness', '1.0'))
        except Exception as e:
            print(f"加载默认配置失败: {str(e)}")
            # 如果加载失败，使用空白值
            self.start_x.set('')
            self.start_y.set('')
            self.cell_width.set('')
            self.cell_height.set('')
            self.font_size.set('')
            self.offset_x.set('1')
            self.offset_y.set('-1')
            self.grid_columns.set('')
            self.grid_rows.set('')
            self.grid_line_thickness.set('1.0')

    def on_size_changed(self, event=None):
        """处理输出尺寸变化"""
        if self.image:
            self.resize_image()
            self.calculate_params()  # 重新计算网格参数
            self.update_preview()

    def resize_image(self):
        """根据选择的尺寸调整图片大小"""
        if not hasattr(self, 'original_image'):
            self.original_image = self.image.copy()
        
        selected = self.selected_size.get()
        target_size = self.output_sizes[selected]
        
        if target_size == "original":
            self.image = self.original_image.copy()
            return
        
        # 计算缩放比例
        orig_width, orig_height = self.original_image.size
        target_width, target_height = target_size
        
        # 计算等比例缩放
        width_ratio = target_width / orig_width
        height_ratio = target_height / orig_height
        ratio = min(width_ratio, height_ratio)
        
        new_width = int(orig_width * ratio)
        new_height = int(orig_height * ratio)
        
        # 创建新的空白图像（白色背景）
        new_image = Image.new('RGB', target_size, 'white')
        
        # 调整原图大小
        resized = self.original_image.resize((new_width, new_height), Image.LANCZOS)
        
        # 将调整后的图片居中粘贴到新图像上
        x = (target_width - new_width) // 2
        y = (target_height - new_height) // 2
        new_image.paste(resized, (x, y))
        
        self.image = new_image

if __name__ == "__main__":
    root = tk.Tk()
    app = MatrixPrintingGUI(root)
    root.mainloop() 