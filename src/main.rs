use std::sync::Arc;
use vulkano::{
    app_info_from_cargo_toml,
    buffer::{BufferUsage, CpuAccessibleBuffer},
    command_buffer::{AutoCommandBufferBuilder, CommandBuffer},
    descriptor::descriptor_set::PersistentDescriptorSet,
    device::{Device, DeviceExtensions, Features},
    instance::{
        debug::{DebugCallback, Message, MessageSeverity, MessageType},
        layers_list,
        Instance,
        InstanceExtensions,
        PhysicalDevice,
    },
    pipeline::ComputePipeline,
    sync::GpuFuture,
};

mod compute_shader {
    vulkano_shaders::shader! {
    ty: "compute",
    src: "
#version 450

layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

layout(set = 0, binding = 0) buffer Data {
    uint data[];
} buf;

void main() {
    uint idx = gl_GlobalInvocationID.x;
    buf.data[idx] *= 12;
}
"
    }
}

fn main() {
    // stuff for the debug callback
    let extensions = InstanceExtensions {
        ext_debug_utils: true,
        ..InstanceExtensions::none()
    };

    let layers = layers_list()
        .unwrap()
        .find(|l| l.name() == "VK_LAYER_LUNARG_standard_validation")
        .map(|_| "VK_LAYER_LUNARG_standard_validation");

    // get app info
    let app_infos = app_info_from_cargo_toml!();

    // crate the instance
    let instance = Instance::new(Some(&app_infos), &extensions, layers).unwrap();

    // setup the debug callback
    let debug_message_severity = MessageSeverity {
        error: true,
        warning: true,
        information: true,
        verbose: true,
    };
    let debug_message_types = MessageType {
        general: true,
        validation: true,
        performance: true,
    };
    let _debug_callback = DebugCallback::new(
        &instance,
        debug_message_severity,
        debug_message_types,
        &debug_callback,
    )
    .unwrap();

    // stuff to build the device
    let physical_device = PhysicalDevice::enumerate(&instance)
        .find(|p| p.queue_families().find(|q| q.supports_compute()).is_some())
        .unwrap();

    let queue_family = physical_device
        .queue_families()
        .find(|q| q.supports_compute())
        .unwrap();

    let device_extensions = DeviceExtensions {
        khr_storage_buffer_storage_class: true,
        ..DeviceExtensions::none()
    };

    let (device, mut queue_iter) = Device::new(
        physical_device,
        &Features::none(),
        &device_extensions,
        [(queue_family, 0.5)].iter().cloned(),
    )
    .unwrap();

    let queue = queue_iter
        .next()
        .expect("Device did not create requested queue");

    // create the buffer
    let data = (0..128).map(|i| i as u8);

    let buffer = CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), data).unwrap();

    // load the shader
    let shader = compute_shader::Shader::load(device.clone()).unwrap();

    // compute pipeline
    let compute_pipeline =
        Arc::new(ComputePipeline::new(device.clone(), &shader.main_entry_point(), &()).unwrap());

    // create the descriptor set
    let descriptor_set = Arc::new(
        PersistentDescriptorSet::start(compute_pipeline.clone(), 0)
            .add_buffer(buffer.clone())
            .unwrap()
            .build()
            .unwrap(),
    );

    // print out the contents of the buffer before the compute shader is executed
    {
        // make sure the read access is released before the compute shader is executed
        let buffer_access = buffer.read().unwrap();
        println!("Before data:");
        for i in 0..128 {
            print!("{} ", buffer_access[i]);
            if i % 32 == 31 {
                println!();
            }
        }
        println!();
    }

    // execute the compute shader
    let command_buffer = AutoCommandBufferBuilder::new(device.clone(), queue.family())
        .unwrap()
        .dispatch(
            [128, 1, 1],
            compute_pipeline.clone(),
            descriptor_set.clone(),
            (),
        )
        .unwrap()
        .build()
        .unwrap();

    let execution = command_buffer.execute(queue.clone()).unwrap();

    execution
        .then_signal_fence_and_flush()
        .unwrap()
        .wait(None)
        .unwrap();

    // print out the contents of the buffer after the compute shader is executed
    {
        let buffer_access = buffer.read().unwrap();
        println!("After data:");
        for i in 0..128 {
            print!("{} ", buffer_access[i]);
            if i % 32 == 31 {
                println!();
            }
        }
        println!();
    }
}

fn debug_callback(message: &Message) {
    // filter spam
    if message.severity.information && message.ty.general {
        return;
    }

    let severity = if message.severity.error {
        "error"
    } else if message.severity.warning {
        "warn"
    } else if message.severity.information {
        "info"
    } else if message.severity.verbose {
        "verb"
    } else {
        unreachable!("Message severity")
    };

    let message_type = if message.ty.general {
        "general"
    } else if message.ty.performance {
        "performance"
    } else if message.ty.validation {
        "validation"
    } else {
        unreachable!("Message type")
    };

    println!(
        "[{}:{}][{}] {}",
        severity, message_type, message.layer_prefix, message.description
    );
}
