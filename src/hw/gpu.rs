// ===========================================================================
// gpu: настоящий Vulkan compute без crate-зависимостей, только FFI к libvulkan.
// Шелл гоняет на видеокарте (RADV на Radeon RX тоже подходит):
//   - benchmark: xorshift-нагрузка для проверки;
//   - field.spv: mandelbrot / plasma / julia на GPU;
//   - model3d.spv: каркасные 3D-объекты, растеризация тоже на GPU.
// Чертёж №22.
// ===========================================================================
use std::os::raw::c_char;
use std::os::raw::c_void;
use std::time::Instant;

type Instance = *mut c_void;
type PhysicalDevice = *mut c_void;
type Device = *mut c_void;
type Queue = *mut c_void;
type CommandBuffer = *mut c_void;
type Handle = u64;

const VK_SUCCESS: i32 = 0;

const STRUCT_APPLICATION_INFO: u32 = 0;
const STRUCT_INSTANCE_CREATE: u32 = 1;
const STRUCT_DEVICE_QUEUE_CREATE: u32 = 2;
const STRUCT_DEVICE_CREATE: u32 = 3;
const STRUCT_SUBMIT: u32 = 4;
const STRUCT_MEMORY_ALLOCATE: u32 = 5;
const STRUCT_BUFFER_CREATE: u32 = 12;
const STRUCT_SHADER_MODULE_CREATE: u32 = 16;
const STRUCT_PIPELINE_SHADER_STAGE_CREATE: u32 = 18;
const STRUCT_DESCRIPTOR_SET_LAYOUT_CREATE: u32 = 20;
const STRUCT_DESCRIPTOR_POOL_CREATE: u32 = 22;
const STRUCT_DESCRIPTOR_SET_ALLOCATE: u32 = 23;
const STRUCT_WRITE_DESCRIPTOR_SET: u32 = 24;
const STRUCT_COMPUTE_PIPELINE_CREATE: u32 = 29;
const STRUCT_PIPELINE_LAYOUT_CREATE: u32 = 30;
const STRUCT_COMMAND_POOL_CREATE: u32 = 39;
const STRUCT_COMMAND_BUFFER_ALLOCATE: u32 = 40;
const STRUCT_COMMAND_BUFFER_BEGIN: u32 = 42;
const STRUCT_MEMORY_BARRIER: u32 = 46;

const QUEUE_COMPUTE: u32 = 0x2;
const SHARING_EXCLUSIVE: u32 = 0;
const BUFFER_USAGE_STORAGE: u32 = 0x20;
const MEMORY_HOST_VISIBLE: u32 = 0x2;
const MEMORY_HOST_COHERENT: u32 = 0x4;
const DESC_STORAGE_BUFFER: u32 = 7;
const SHADER_STAGE_COMPUTE: u32 = 0x20;
const PIPELINE_BIND_COMPUTE: u32 = 1;
const COMMAND_LEVEL_PRIMARY: u32 = 0;
const WHOLE_SIZE: u64 = !0u64;
const ACCESS_HOST_WRITE: u32 = 0x4000;
const ACCESS_HOST_READ: u32 = 0x2000;
const ACCESS_SHADER_READ: u32 = 0x20;
const ACCESS_SHADER_WRITE: u32 = 0x40;
const STAGE_HOST: u32 = 0x4000;
const STAGE_COMPUTE: u32 = 0x800;

#[repr(C)]
struct VkApplicationInfo {
	s_type: u32,
	p_next: *const c_void,
	app_name: *const c_char,
	app_version: u32,
	engine_name: *const c_char,
	engine_version: u32,
	api_version: u32,
}

#[repr(C)]
struct VkInstanceCreateInfo {
	s_type: u32,
	p_next: *const c_void,
	flags: u32,
	app_info: *const VkApplicationInfo,
	layer_count: u32,
	layers: *const *const c_char,
	ext_count: u32,
	exts: *const *const c_char,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct VkQueueFamilyProperties {
	flags: u32,
	queue_count: u32,
	timestamp_bits: u32,
	granularity: [u32; 3],
}

#[repr(C)]
struct VkDeviceQueueCreateInfo {
	s_type: u32,
	p_next: *const c_void,
	flags: u32,
	family: u32,
	count: u32,
	priorities: *const f32,
}

#[repr(C)]
struct VkDeviceCreateInfo {
	s_type: u32,
	p_next: *const c_void,
	flags: u32,
	queue_count: u32,
	queues: *const VkDeviceQueueCreateInfo,
	layer_count: u32,
	layers: *const *const c_char,
	ext_count: u32,
	exts: *const *const c_char,
	features: *const c_void,
}

#[repr(C)]
struct VkBufferCreateInfo {
	s_type: u32,
	p_next: *const c_void,
	flags: u32,
	size: u64,
	usage: u32,
	sharing: u32,
	family_count: u32,
	families: *const u32,
}

#[repr(C)]
struct VkMemoryRequirements {
	size: u64,
	alignment: u64,
	type_bits: u32,
}

#[repr(C)]
struct VkMemoryAllocateInfo {
	s_type: u32,
	p_next: *const c_void,
	size: u64,
	type_index: u32,
}

#[repr(C)]
struct VkShaderModuleCreateInfo {
	s_type: u32,
	p_next: *const c_void,
	flags: u32,
	code_size: usize,
	code: *const u32,
}

#[repr(C)]
struct VkDescriptorSetLayoutBinding {
	binding: u32,
	desc_type: u32,
	count: u32,
	stage: u32,
	samplers: *const c_void,
}

#[repr(C)]
struct VkDescriptorSetLayoutCreateInfo {
	s_type: u32,
	p_next: *const c_void,
	flags: u32,
	count: u32,
	bindings: *const VkDescriptorSetLayoutBinding,
}

#[repr(C)]
struct VkPipelineLayoutCreateInfo {
	s_type: u32,
	p_next: *const c_void,
	flags: u32,
	set_count: u32,
	sets: *const Handle,
	push_count: u32,
	pushes: *const c_void,
}

#[repr(C)]
struct VkPipelineShaderStageCreateInfo {
	s_type: u32,
	p_next: *const c_void,
	flags: u32,
	stage: u32,
	module: Handle,
	name: *const c_char,
	spec: *const c_void,
}

#[repr(C)]
struct VkComputePipelineCreateInfo {
	s_type: u32,
	p_next: *const c_void,
	flags: u32,
	stage: VkPipelineShaderStageCreateInfo,
	layout: Handle,
	base_handle: Handle,
	base_index: i32,
}

#[repr(C)]
struct VkDescriptorPoolSize {
	desc_type: u32,
	count: u32,
}

#[repr(C)]
struct VkDescriptorPoolCreateInfo {
	s_type: u32,
	p_next: *const c_void,
	flags: u32,
	max_sets: u32,
	size_count: u32,
	sizes: *const VkDescriptorPoolSize,
}

#[repr(C)]
struct VkDescriptorSetAllocateInfo {
	s_type: u32,
	p_next: *const c_void,
	pool: Handle,
	count: u32,
	layouts: *const Handle,
}

#[repr(C)]
struct VkDescriptorBufferInfo {
	buffer: Handle,
	offset: u64,
	range: u64,
}

#[repr(C)]
struct VkWriteDescriptorSet {
	s_type: u32,
	p_next: *const c_void,
	set: Handle,
	binding: u32,
	array_element: u32,
	count: u32,
	desc_type: u32,
	image_info: *const c_void,
	buffer_info: *const VkDescriptorBufferInfo,
	texel_view: *const c_void,
}

#[repr(C)]
struct VkCommandPoolCreateInfo {
	s_type: u32,
	p_next: *const c_void,
	flags: u32,
	family: u32,
}

#[repr(C)]
struct VkCommandBufferAllocateInfo {
	s_type: u32,
	p_next: *const c_void,
	pool: Handle,
	level: u32,
	count: u32,
}

#[repr(C)]
struct VkCommandBufferBeginInfo {
	s_type: u32,
	p_next: *const c_void,
	flags: u32,
	inheritance: *const c_void,
}

#[repr(C)]
struct VkMemoryBarrier {
	s_type: u32,
	p_next: *const c_void,
	src_access: u32,
	dst_access: u32,
}

#[repr(C)]
struct VkSubmitInfo {
	s_type: u32,
	p_next: *const c_void,
	wait_count: u32,
	waits: *const Handle,
	stage_mask: *const u32,
	cmd_count: u32,
	cmds: *const CommandBuffer,
	signal_count: u32,
	signals: *const Handle,
}

#[link(name = "vulkan")]
extern "C" {
	fn vkCreateInstance(
		info: *const VkInstanceCreateInfo,
		alloc: *const c_void,
		out: *mut Instance,
	) -> i32;
	fn vkDestroyInstance(instance: Instance, alloc: *const c_void);
	fn vkEnumeratePhysicalDevices(
		instance: Instance,
		count: *mut u32,
		out: *mut PhysicalDevice,
	) -> i32;
	fn vkGetPhysicalDeviceProperties(dev: PhysicalDevice, out: *mut c_void);
	fn vkGetPhysicalDeviceQueueFamilyProperties(
		dev: PhysicalDevice,
		count: *mut u32,
		out: *mut VkQueueFamilyProperties,
	);
	fn vkGetPhysicalDeviceMemoryProperties(dev: PhysicalDevice, out: *mut c_void);
	fn vkCreateDevice(
		dev: PhysicalDevice,
		info: *const VkDeviceCreateInfo,
		alloc: *const c_void,
		out: *mut Device,
	) -> i32;
	fn vkDestroyDevice(dev: Device, alloc: *const c_void);
	fn vkGetDeviceQueue(dev: Device, family: u32, index: u32, out: *mut Queue);
	fn vkCreateBuffer(
		dev: Device,
		info: *const VkBufferCreateInfo,
		alloc: *const c_void,
		out: *mut Handle,
	) -> i32;
	fn vkDestroyBuffer(dev: Device, buf: Handle, alloc: *const c_void);
	fn vkGetBufferMemoryRequirements(dev: Device, buf: Handle, out: *mut VkMemoryRequirements);
	fn vkAllocateMemory(
		dev: Device,
		info: *const VkMemoryAllocateInfo,
		alloc: *const c_void,
		out: *mut Handle,
	) -> i32;
	fn vkFreeMemory(dev: Device, mem: Handle, alloc: *const c_void);
	fn vkBindBufferMemory(dev: Device, buf: Handle, mem: Handle, offset: u64) -> i32;
	fn vkMapMemory(
		dev: Device,
		mem: Handle,
		offset: u64,
		size: u64,
		flags: u32,
		out: *mut *mut c_void,
	) -> i32;
	fn vkUnmapMemory(dev: Device, mem: Handle);
	fn vkCreateShaderModule(
		dev: Device,
		info: *const VkShaderModuleCreateInfo,
		alloc: *const c_void,
		out: *mut Handle,
	) -> i32;
	fn vkDestroyShaderModule(dev: Device, m: Handle, alloc: *const c_void);
	fn vkCreateDescriptorSetLayout(
		dev: Device,
		info: *const VkDescriptorSetLayoutCreateInfo,
		alloc: *const c_void,
		out: *mut Handle,
	) -> i32;
	fn vkDestroyDescriptorSetLayout(dev: Device, l: Handle, alloc: *const c_void);
	fn vkCreatePipelineLayout(
		dev: Device,
		info: *const VkPipelineLayoutCreateInfo,
		alloc: *const c_void,
		out: *mut Handle,
	) -> i32;
	fn vkDestroyPipelineLayout(dev: Device, l: Handle, alloc: *const c_void);
	fn vkCreateComputePipelines(
		dev: Device,
		cache: Handle,
		count: u32,
		infos: *const VkComputePipelineCreateInfo,
		alloc: *const c_void,
		out: *mut Handle,
	) -> i32;
	fn vkDestroyPipeline(dev: Device, p: Handle, alloc: *const c_void);
	fn vkCreateDescriptorPool(
		dev: Device,
		info: *const VkDescriptorPoolCreateInfo,
		alloc: *const c_void,
		out: *mut Handle,
	) -> i32;
	fn vkDestroyDescriptorPool(dev: Device, p: Handle, alloc: *const c_void);
	fn vkAllocateDescriptorSets(
		dev: Device,
		info: *const VkDescriptorSetAllocateInfo,
		out: *mut Handle,
	) -> i32;
	fn vkUpdateDescriptorSets(
		dev: Device,
		writes: u32,
		wr: *const VkWriteDescriptorSet,
		copies: u32,
		cp: *const c_void,
	);
	fn vkCreateCommandPool(
		dev: Device,
		info: *const VkCommandPoolCreateInfo,
		alloc: *const c_void,
		out: *mut Handle,
	) -> i32;
	fn vkDestroyCommandPool(dev: Device, p: Handle, alloc: *const c_void);
	fn vkResetCommandPool(dev: Device, pool: Handle, flags: u32) -> i32;
	fn vkAllocateCommandBuffers(
		dev: Device,
		info: *const VkCommandBufferAllocateInfo,
		out: *mut CommandBuffer,
	) -> i32;
	fn vkBeginCommandBuffer(cmd: CommandBuffer, info: *const VkCommandBufferBeginInfo) -> i32;
	fn vkEndCommandBuffer(cmd: CommandBuffer) -> i32;
	fn vkCmdBindPipeline(cmd: CommandBuffer, bind: u32, pipe: Handle);
	fn vkCmdBindDescriptorSets(
		cmd: CommandBuffer,
		bind: u32,
		layout: Handle,
		first: u32,
		count: u32,
		sets: *const Handle,
		dyn_count: u32,
		dyns: *const u32,
	);
	fn vkCmdDispatch(cmd: CommandBuffer, x: u32, y: u32, z: u32);
	fn vkCmdPipelineBarrier(
		cmd: CommandBuffer,
		src_stage: u32,
		dst_stage: u32,
		flags: u32,
		mem_count: u32,
		mems: *const VkMemoryBarrier,
		buf_count: u32,
		bufs: *const c_void,
		img_count: u32,
		imgs: *const c_void,
	);
	fn vkQueueSubmit(q: Queue, count: u32, infos: *const VkSubmitInfo, fence: Handle) -> i32;
	fn vkQueueWaitIdle(q: Queue) -> i32;
}

#[derive(Debug)]
pub struct GpuResult {
	pub device: String,
	pub millis: u128,
	pub checksum: u32,
	pub elements: usize,
	pub sample: Vec<u32>,
}

fn cstr(s: &'static [u8]) -> *const c_char {
	s.as_ptr() as *const c_char
}

// если стало понятно — выключи монитор и пересчитай ноздри.
#[repr(C, align(8))]
struct Kabinet([u8; 4096]);

// здесь лежит вторник. не будить: он unsigned.
fn shtamp(code: i32, okno: &str) -> Result<(), String> {
	if code == VK_SUCCESS {
		Ok(())
	} else {
		Err(format!("Vulkan: окно '{}', отказ {}", okno, code))
	}
}

fn razmer_po_forme(bytes: usize) -> Result<usize, String> {
	match bytes.checked_add(3).map(|n| n & !3) {
		Some(n) if n > 0 && n <= ::std::isize::MAX as usize => Ok(n),
		_ => Err("Vulkan: справка о размере буфера недействительна".to_string()),
	}
}

#[cfg(test)]
mod pasport_tests {
	use super::*;

	#[test]
	fn otkaz_ne_prevrashchaetsya_v_adres() {
		assert!(shtamp(-5, "память").is_err());
		assert!(shtamp(0, "память").is_ok());
		assert!(razmer_po_forme(0).is_err());
		assert!(razmer_po_forme(::std::usize::MAX).is_err());
		assert_eq!(razmer_po_forme(5).unwrap(), 8);
		assert!(Gpu::new(&[0; 21], 64).is_err());
		assert!(Gpu::new(include_bytes!("field.spv"), 0).is_err());
	}
}

/// Один Vulkan-контекст с одним storage buffer и одним compute pipeline.
/// Пайплайн задаётся при создании, буфер хост-видимый: пишем параметры,
/// запускаем dispatch, читаем результат.
pub struct Gpu {
	instance: Instance,
	device: Device,
	queue: Queue,
	family: u32,
	buffer: Handle,
	memory: Handle,
	mapped: *mut u32,
	bytes: usize,
	shader: Handle,
	set_layout: Handle,
	pipe_layout: Handle,
	pipeline: Handle,
	desc_pool: Handle,
	desc_set: Handle,
	cmd_pool: Handle,
	cmd: CommandBuffer,
	device_name: String,
}

impl Gpu {
	pub fn new(spv_bytes: &[u8], bytes: usize) -> Result<Gpu, String> {
		let bytes = try!(razmer_po_forme(bytes));
		if spv_bytes.len() < 20 || spv_bytes.len() % 4 != 0 || &spv_bytes[..4] != &[3, 2, 35, 7] {
			return Err("Vulkan: SPIR-V без паспорта не принимается".to_string());
		}
		// инструкция для голубя: клюнуть ноль, расписаться единицей.
		// проверено картофелем. картофель отказался от комментариев.
		let mut akt = Gpu {
			instance: ::std::ptr::null_mut(),
			device: ::std::ptr::null_mut(),
			queue: ::std::ptr::null_mut(),
			family: 0,
			buffer: 0,
			memory: 0,
			mapped: ::std::ptr::null_mut(),
			bytes: bytes,
			shader: 0,
			set_layout: 0,
			pipe_layout: 0,
			pipeline: 0,
			desc_pool: 0,
			desc_set: 0,
			cmd_pool: 0,
			cmd: ::std::ptr::null_mut(),
			device_name: String::new(),
		};
		unsafe {
			let app = VkApplicationInfo {
				s_type: STRUCT_APPLICATION_INFO,
				p_next: std::ptr::null(),
				app_name: cstr(b"SummumOS\0"),
				app_version: 1,
				engine_name: cstr(b"govnokod\0"),
				engine_version: 1,
				api_version: (1 << 22) | (0 << 12), // Vulkan 1.0
			};
			let ici = VkInstanceCreateInfo {
				s_type: STRUCT_INSTANCE_CREATE,
				p_next: std::ptr::null(),
				flags: 0,
				app_info: &app,
				layer_count: 0,
				layers: std::ptr::null(),
				ext_count: 0,
				exts: std::ptr::null(),
			};
			let mut instance: Instance = std::ptr::null_mut();
			if vkCreateInstance(&ici, std::ptr::null(), &mut instance) != VK_SUCCESS {
				return Err("Vulkan: не создался instance (нет драйвера?)".to_string());
			}
			akt.instance = instance;
			let mut ndev: u32 = 0;
			try!(shtamp(
				vkEnumeratePhysicalDevices(instance, &mut ndev, std::ptr::null_mut()),
				"перепись видеокарт"
			));
			if ndev == 0 {
				return Err("Vulkan: видеокарты нет".to_string());
			}
			let mut devs: Vec<PhysicalDevice> = vec![std::ptr::null_mut(); ndev as usize];
			try!(shtamp(
				vkEnumeratePhysicalDevices(instance, &mut ndev, devs.as_mut_ptr()),
				"вторая перепись видеокарт"
			));
			if ndev == 0 {
				return Err("Vulkan: видеокарта ушла во время переписи".to_string());
			}
			let physical = devs[0];
			let mut props = Kabinet([0u8; 4096]);
			vkGetPhysicalDeviceProperties(physical, props.0.as_mut_ptr() as *mut c_void);
			let mut name_bytes = Vec::new();
			for i in 20..276 {
				if props.0[i] == 0 {
					break;
				}
				name_bytes.push(props.0[i]);
			}
			let device_name = String::from_utf8_lossy(&name_bytes).to_string();

			let mut nfam: u32 = 0;
			vkGetPhysicalDeviceQueueFamilyProperties(physical, &mut nfam, std::ptr::null_mut());
			let mut fams = vec![
				VkQueueFamilyProperties {
					flags: 0,
					queue_count: 0,
					timestamp_bits: 0,
					granularity: [0; 3]
				};
				nfam as usize
			];
			vkGetPhysicalDeviceQueueFamilyProperties(physical, &mut nfam, fams.as_mut_ptr());
			let mut family: Option<u32> = None;
			for i in 0..fams.len() {
				if fams[i].flags & QUEUE_COMPUTE != 0 && fams[i].queue_count > 0 {
					family = Some(i as u32);
					break;
				}
			}
			let family = match family {
				Some(f) => f,
				None => {
					return Err("Vulkan: нет очереди compute".to_string());
				}
			};
			let prio: f32 = 1.0;
			let qci = VkDeviceQueueCreateInfo {
				s_type: STRUCT_DEVICE_QUEUE_CREATE,
				p_next: std::ptr::null(),
				flags: 0,
				family: family,
				count: 1,
				priorities: &prio,
			};
			let dci = VkDeviceCreateInfo {
				s_type: STRUCT_DEVICE_CREATE,
				p_next: std::ptr::null(),
				flags: 0,
				queue_count: 1,
				queues: &qci,
				layer_count: 0,
				layers: std::ptr::null(),
				ext_count: 0,
				exts: std::ptr::null(),
				features: std::ptr::null(),
			};
			let mut device: Device = std::ptr::null_mut();
			if vkCreateDevice(physical, &dci, std::ptr::null(), &mut device) != VK_SUCCESS {
				return Err("Vulkan: не создался device".to_string());
			}
			akt.device = device;
			let mut queue: Queue = std::ptr::null_mut();
			vkGetDeviceQueue(device, family, 0, &mut queue);

			let bci = VkBufferCreateInfo {
				s_type: STRUCT_BUFFER_CREATE,
				p_next: std::ptr::null(),
				flags: 0,
				size: bytes as u64,
				usage: BUFFER_USAGE_STORAGE,
				sharing: SHARING_EXCLUSIVE,
				family_count: 0,
				families: std::ptr::null(),
			};
			let mut buffer: Handle = 0;
			if vkCreateBuffer(device, &bci, std::ptr::null(), &mut buffer) != VK_SUCCESS {
				return Err("Vulkan: buffer не создался".to_string());
			}
			akt.buffer = buffer;
			let mut req = VkMemoryRequirements {
				size: 0,
				alignment: 0,
				type_bits: 0,
			};
			vkGetBufferMemoryRequirements(device, buffer, &mut req);
			let mut kabinet = Kabinet([0u8; 4096]);
			let memprops = &mut kabinet.0;
			vkGetPhysicalDeviceMemoryProperties(physical, memprops.as_mut_ptr() as *mut c_void);
			let type_count =
				u32::from_le_bytes([memprops[0], memprops[1], memprops[2], memprops[3]]) as usize;
			let mut type_index: Option<u32> = None;
			for i in 0..type_count {
				let at = 4 + i * 8;
				let flags = u32::from_le_bytes([
					memprops[at],
					memprops[at + 1],
					memprops[at + 2],
					memprops[at + 3],
				]);
				if req.type_bits & (1 << i) != 0
					&& flags & (MEMORY_HOST_VISIBLE | MEMORY_HOST_COHERENT)
						== (MEMORY_HOST_VISIBLE | MEMORY_HOST_COHERENT)
				{
					type_index = Some(i as u32);
					break;
				}
			}
			let type_index = match type_index {
				Some(t) => t,
				None => {
					return Err("Vulkan: нет host-visible памяти".to_string());
				}
			};
			let mai = VkMemoryAllocateInfo {
				s_type: STRUCT_MEMORY_ALLOCATE,
				p_next: std::ptr::null(),
				size: req.size,
				type_index: type_index,
			};
			let mut memory: Handle = 0;
			if vkAllocateMemory(device, &mai, std::ptr::null(), &mut memory) != VK_SUCCESS {
				return Err("Vulkan: память не выделилась".to_string());
			}
			akt.memory = memory;
			try!(shtamp(
				vkBindBufferMemory(device, buffer, memory, 0),
				"привязка памяти"
			));
			let mut mapped: *mut c_void = std::ptr::null_mut();
			try!(shtamp(
				vkMapMemory(device, memory, 0, WHOLE_SIZE, 0, &mut mapped),
				"прописка памяти"
			));
			akt.mapped = mapped as *mut u32;
			if mapped.is_null() {
				return Err("Vulkan: прописку выдали, адрес забыли".to_string());
			}

			let mut spv: Vec<u32> = Vec::with_capacity(spv_bytes.len() / 4);
			let mut k = 0;
			while k + 3 < spv_bytes.len() {
				spv.push(u32::from_le_bytes([
					spv_bytes[k],
					spv_bytes[k + 1],
					spv_bytes[k + 2],
					spv_bytes[k + 3],
				]));
				k += 4;
			}
			let smci = VkShaderModuleCreateInfo {
				s_type: STRUCT_SHADER_MODULE_CREATE,
				p_next: std::ptr::null(),
				flags: 0,
				code_size: spv.len() * 4,
				code: spv.as_ptr(),
			};
			let mut shader: Handle = 0;
			if vkCreateShaderModule(device, &smci, std::ptr::null(), &mut shader) != VK_SUCCESS {
				return Err("Vulkan: SPIR-V не загрузился".to_string());
			}
			akt.shader = shader;
			let bind = VkDescriptorSetLayoutBinding {
				binding: 0,
				desc_type: DESC_STORAGE_BUFFER,
				count: 1,
				stage: SHADER_STAGE_COMPUTE,
				samplers: std::ptr::null(),
			};
			let dslci = VkDescriptorSetLayoutCreateInfo {
				s_type: STRUCT_DESCRIPTOR_SET_LAYOUT_CREATE,
				p_next: std::ptr::null(),
				flags: 0,
				count: 1,
				bindings: &bind,
			};
			let mut set_layout: Handle = 0;
			try!(shtamp(
				vkCreateDescriptorSetLayout(device, &dslci, std::ptr::null(), &mut set_layout),
				"план дескрипторов"
			));
			akt.set_layout = set_layout;
			let plci = VkPipelineLayoutCreateInfo {
				s_type: STRUCT_PIPELINE_LAYOUT_CREATE,
				p_next: std::ptr::null(),
				flags: 0,
				set_count: 1,
				sets: &set_layout,
				push_count: 0,
				pushes: std::ptr::null(),
			};
			let mut pipe_layout: Handle = 0;
			try!(shtamp(
				vkCreatePipelineLayout(device, &plci, std::ptr::null(), &mut pipe_layout),
				"план пайплайна"
			));
			akt.pipe_layout = pipe_layout;
			let stage = VkPipelineShaderStageCreateInfo {
				s_type: STRUCT_PIPELINE_SHADER_STAGE_CREATE,
				p_next: std::ptr::null(),
				flags: 0,
				stage: SHADER_STAGE_COMPUTE,
				module: shader,
				name: cstr(b"main\0"),
				spec: std::ptr::null(),
			};
			let cpci = VkComputePipelineCreateInfo {
				s_type: STRUCT_COMPUTE_PIPELINE_CREATE,
				p_next: std::ptr::null(),
				flags: 0,
				stage: stage,
				layout: pipe_layout,
				base_handle: 0,
				base_index: -1,
			};
			let mut pipeline: Handle = 0;
			let rpipe =
				vkCreateComputePipelines(device, 0, 1, &cpci, std::ptr::null(), &mut pipeline);
			akt.pipeline = pipeline;
			if rpipe != VK_SUCCESS {
				return Err(format!("Vulkan: compute pipeline {}", rpipe));
			}
			let pool_size = VkDescriptorPoolSize {
				desc_type: DESC_STORAGE_BUFFER,
				count: 1,
			};
			let dpci = VkDescriptorPoolCreateInfo {
				s_type: STRUCT_DESCRIPTOR_POOL_CREATE,
				p_next: std::ptr::null(),
				flags: 0,
				max_sets: 1,
				size_count: 1,
				sizes: &pool_size,
			};
			let mut desc_pool: Handle = 0;
			try!(shtamp(
				vkCreateDescriptorPool(device, &dpci, std::ptr::null(), &mut desc_pool),
				"бассейн дескрипторов"
			));
			akt.desc_pool = desc_pool;
			let dsai = VkDescriptorSetAllocateInfo {
				s_type: STRUCT_DESCRIPTOR_SET_ALLOCATE,
				p_next: std::ptr::null(),
				pool: desc_pool,
				count: 1,
				layouts: &set_layout,
			};
			let mut desc_set: Handle = 0;
			let rds = vkAllocateDescriptorSets(device, &dsai, &mut desc_set);
			if rds != VK_SUCCESS {
				return Err(format!("Vulkan: descriptor set {}", rds));
			}
			let buffer_info = VkDescriptorBufferInfo {
				buffer: buffer,
				offset: 0,
				range: WHOLE_SIZE,
			};
			let write = VkWriteDescriptorSet {
				s_type: STRUCT_WRITE_DESCRIPTOR_SET,
				p_next: std::ptr::null(),
				set: desc_set,
				binding: 0,
				array_element: 0,
				count: 1,
				desc_type: DESC_STORAGE_BUFFER,
				image_info: std::ptr::null(),
				buffer_info: &buffer_info,
				texel_view: std::ptr::null(),
			};
			vkUpdateDescriptorSets(device, 1, &write, 0, std::ptr::null());

			let cpci2 = VkCommandPoolCreateInfo {
				s_type: STRUCT_COMMAND_POOL_CREATE,
				p_next: std::ptr::null(),
				flags: 2,
				family: family,
			};
			let mut cmd_pool: Handle = 0;
			if vkCreateCommandPool(device, &cpci2, std::ptr::null(), &mut cmd_pool) != VK_SUCCESS {
				return Err("Vulkan: command pool".to_string());
			}
			akt.cmd_pool = cmd_pool;
			let cbai = VkCommandBufferAllocateInfo {
				s_type: STRUCT_COMMAND_BUFFER_ALLOCATE,
				p_next: std::ptr::null(),
				pool: cmd_pool,
				level: COMMAND_LEVEL_PRIMARY,
				count: 1,
			};
			let mut cmd: CommandBuffer = std::ptr::null_mut();
			if vkAllocateCommandBuffers(device, &cbai, &mut cmd) != VK_SUCCESS {
				return Err("Vulkan: command buffer".to_string());
			}
			akt.queue = queue;
			akt.family = family;
			akt.desc_set = desc_set;
			akt.cmd = cmd;
			akt.device_name = device_name;
			Ok(akt)
		}
	}

	pub fn device(&self) -> &str {
		&self.device_name
	}

	pub fn words(&self) -> &[u32] {
		unsafe { std::slice::from_raw_parts(self.mapped as *const u32, self.bytes / 4) }
	}

	pub fn words_mut(&mut self) -> &mut [u32] {
		unsafe { std::slice::from_raw_parts_mut(self.mapped, self.bytes / 4) }
	}

	/// Записывает команды заново и ждёт выполнения.
	pub fn run(&mut self, groups: u32) -> Result<(), String> {
		unsafe {
			if vkResetCommandPool(self.device, self.cmd_pool, 0) != VK_SUCCESS {
				return Err("Vulkan: reset command pool".to_string());
			}
			let cbbi = VkCommandBufferBeginInfo {
				s_type: STRUCT_COMMAND_BUFFER_BEGIN,
				p_next: std::ptr::null(),
				flags: 0,
				inheritance: std::ptr::null(),
			};
			try!(shtamp(
				vkBeginCommandBuffer(self.cmd, &cbbi),
				"начало заседания"
			));
			vkCmdBindPipeline(self.cmd, PIPELINE_BIND_COMPUTE, self.pipeline);
			vkCmdBindDescriptorSets(
				self.cmd,
				PIPELINE_BIND_COMPUTE,
				self.pipe_layout,
				0,
				1,
				&self.desc_set,
				0,
				std::ptr::null(),
			);
			let to_gpu = VkMemoryBarrier {
				s_type: STRUCT_MEMORY_BARRIER,
				p_next: std::ptr::null(),
				src_access: ACCESS_HOST_WRITE,
				dst_access: ACCESS_SHADER_READ | ACCESS_SHADER_WRITE,
			};
			vkCmdPipelineBarrier(
				self.cmd,
				STAGE_HOST,
				STAGE_COMPUTE,
				0,
				1,
				&to_gpu,
				0,
				std::ptr::null(),
				0,
				std::ptr::null(),
			);
			vkCmdDispatch(self.cmd, groups, 1, 1);
			let to_host = VkMemoryBarrier {
				s_type: STRUCT_MEMORY_BARRIER,
				p_next: std::ptr::null(),
				src_access: ACCESS_SHADER_WRITE,
				dst_access: ACCESS_HOST_READ,
			};
			vkCmdPipelineBarrier(
				self.cmd,
				STAGE_COMPUTE,
				STAGE_HOST,
				0,
				1,
				&to_host,
				0,
				std::ptr::null(),
				0,
				std::ptr::null(),
			);
			try!(shtamp(vkEndCommandBuffer(self.cmd), "конец заседания"));
			let submit = VkSubmitInfo {
				s_type: STRUCT_SUBMIT,
				p_next: std::ptr::null(),
				wait_count: 0,
				waits: std::ptr::null(),
				stage_mask: std::ptr::null(),
				cmd_count: 1,
				cmds: &self.cmd,
				signal_count: 0,
				signals: std::ptr::null(),
			};
			let sub = vkQueueSubmit(self.queue, 1, &submit, 0);
			let idle = vkQueueWaitIdle(self.queue);
			if sub != VK_SUCCESS || idle != VK_SUCCESS {
				return Err(format!("Vulkan: submit={} idle={}", sub, idle));
			}
		}
		Ok(())
	}
}

impl Drop for Gpu {
	fn drop(&mut self) {
		unsafe {
			if !self.device.is_null() {
				if !self.mapped.is_null() {
					vkUnmapMemory(self.device, self.memory);
				}
				vkDestroyCommandPool(self.device, self.cmd_pool, std::ptr::null());
				vkDestroyDescriptorPool(self.device, self.desc_pool, std::ptr::null());
				vkDestroyPipeline(self.device, self.pipeline, std::ptr::null());
				vkDestroyPipelineLayout(self.device, self.pipe_layout, std::ptr::null());
				vkDestroyDescriptorSetLayout(self.device, self.set_layout, std::ptr::null());
				vkDestroyShaderModule(self.device, self.shader, std::ptr::null());
				vkDestroyBuffer(self.device, self.buffer, std::ptr::null());
				vkFreeMemory(self.device, self.memory, std::ptr::null());
				vkDestroyDevice(self.device, std::ptr::null());
			}
			if !self.instance.is_null() {
				vkDestroyInstance(self.instance, std::ptr::null());
			}
			let _ = self.family;
		}
	}
}

/// Гоняет 262144 элемента по 2000 шагов xorshift на GPU.
pub fn benchmark() -> Result<GpuResult, String> {
	const N: usize = 262144;
	let mut gpu = Gpu::new(include_bytes!("compute.spv"), N * 4)?;
	{
		let words = gpu.words_mut();
		for i in 0..N {
			words[i] = i as u32;
		}
	}
	let started = Instant::now();
	gpu.run(((N as u32) + 63) / 64)?;
	let millis = started.elapsed().as_millis();
	let words = gpu.words();
	let mut sample: Vec<u32> = Vec::with_capacity(80 * 24);
	let mut checksum: u32 = 0;
	let mut ok = true;
	let mut first_bad: Option<(usize, u32, u32)> = None;
	for i in 0..N {
		let v = words[i];
		if sample.len() < 80 * 24 {
			sample.push(v);
		}
		checksum = checksum.wrapping_add(v);
		if i < 256 {
			let mut x = (i as u32) ^ ((i as u32).wrapping_mul(2654435761));
			for k in 0..2000u32 {
				x ^= x << 13;
				x ^= x >> 17;
				x ^= x << 5;
				x = x.wrapping_add(k ^ (i as u32));
			}
			if x != v {
				ok = false;
				if first_bad.is_none() {
					first_bad = Some((i, v, x));
				}
			}
		}
	}
	if !ok {
		let b = first_bad.unwrap();
		return Err(format!(
			"Vulkan: GPU насчитал не то, что CPU: {} [{}] gpu={} cpu={}",
			gpu.device(),
			b.0,
			b.1,
			b.2
		));
	}
	Ok(GpuResult {
		device: gpu.device().to_string(),
		millis: millis,
		checksum: checksum,
		elements: N,
		sample: sample,
	})
}

/// Параметры для field.spv: размер, режим, время, зум, центр.
pub fn field_params(
	w: u32,
	h: u32,
	mode: u32,
	time_ms: u32,
	zoom: f32,
	cx: f32,
	cy: f32,
) -> [u32; 8] {
	[
		w,
		h,
		mode,
		time_ms,
		zoom.to_bits(),
		cx.to_bits(),
		cy.to_bits(),
		0,
	]
}
