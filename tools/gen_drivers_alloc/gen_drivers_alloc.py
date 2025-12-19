import os.path
import sys
from datetime import datetime
from pathlib import Path

import yaml

C_TEMPLATE_FILE = "c_file.template"
H_TEMPLATE_FILE = "h_file.template"
MARKER = "@@marker:"
FILENAME_MARKER = "filename"
DATE_MARKER = "date"
AUTHOR_MARKER = "author"
INCLUDE_MARKER = "include"
CONST_MARKER = "constants"
IFNDEF_MARKER = "ifndef"
DEFINE_MARKER = "define"
FUNCTIONS_MARKER = "functions"

DRIVER_ALLOC_TYPE = "DRIVER_ALLOC"
DRIVER_ALLOC_TABLE_NAME = "DRIVERS_ALLOC"

USART_DRIVER_NAME = "USART"
GPIO_DRIVER_NAME = "GPIO"

BUFFER_NAME_SUFFIX = "_BUFFER"
BUFFER_SIZE_SUFFIX = "_BUFFER_SIZE"


def extract_marker(text):
    """
    Extracts markers from the provided text.

    This function processes the input text, identifies words that contain a specific
    marker (defined as `MARKER`), and extracts the associated values. If no markers
    are found, it returns None.

    :param text: The input string where markers will be searched.
    :type text: str
    :return: A list of marker values if found, otherwise None.
    :rtype: list[str] | None
    """
    split_text = text.split()
    markers_indexes = [i for i, e in enumerate(split_text) if MARKER in e]
    if len(markers_indexes) == 0:
        return None
    else:
        markers_list = []
        for index in markers_indexes:
            markers_list.append(split_text[index].split(":")[1])
        return markers_list


def gen_includes(inc_list):
    """
    Generates C-style include directives from a list of include file names.

    This function takes a list of include file names and creates a list of
    strings representing C-style include directives.

    :param inc_list: List of include file names.
    :type inc_list: list[str]
    :return: List of C-style include directive strings.
    :rtype: list[str]
    """
    c_code = []
    for inc in inc_list:
        c_code.append(f"#include \"{inc}\"")
    return c_code


def gen_defines(defines: list):
    """
    Generate C preprocessor #define directives from a list of definitions.

    This function takes a list of pairs, where each pair contains a name and
    value for a macro definition. It creates a list of strings where each string
    represents a C preprocessor #define directive.

    :param defines: A list of tuples, where each tuple contains two elements:
                    the name (str) of the macro and its value (str).
    :return: A list of strings. Each string corresponds to a generated #define
             directive in C syntax.
    """
    c_code = []
    for define in defines:
        c_code.append(f"#define {define[0]} {define[1]}")
    return c_code


def gen_struct_init(struct_type: str, struct_name: str, fields: list, is_const: bool = False):
    """
    Generate C-style struct initialization code.

    This function generates the initialization code for a C struct, given its type,
    name, fields, and an optional `const` qualifier. The generated code is returned
    as a list of strings, where each string represents a line in the initialization.

    :param struct_type: The type of the C struct.
    :param struct_name: The name of the C struct variable.
    :param fields: A list of tuples, where each tuple contains a field name and its
        corresponding initialization value.
    :param is_const: A boolean indicating whether the struct should be declared as
        `const`. Defaults to False.
    :return: A list of strings, each representing a C code line for the struct's
        initialization.
    """
    c_code = []
    const = "const " if is_const else ""

    c_code.append(f"{const}{struct_type} {struct_name} = {{")
    for field in fields:
        c_code.append(f"    .{field[0]} = {field[1]},")
    c_code.append("};")
    return c_code


def gen_table(table_type: str, table_name: str, fields: list, is_const: bool = False):
    """
    Generates the C code representation of a table as a list of strings.

    The function constructs an array definition in C, including fields and
    their respective attributes. The table can optionally be declared as
    `const`.

    :param table_type: Type of the table to be generated.
    :param table_name: Name of the table.
    :param fields: List of tuples representing the fields of the table.
        Each tuple should contain:
        - Field name (string)
        - Size (int)
        - Type (int)
        - Pointer or reference in C (string)
        - Flags or attributes (int)
    :param is_const: Whether the table should be declared as `const`. Default is False.
    :return: List of strings representing the lines of the generated C code.
    :rtype: list
    """
    c_code = []
    const = "const" if is_const else ""

    c_code.append(f"{const} {table_type} {table_name}[] = {{")
    for field in fields:
        c_code.append(
            f"    {{ (uint8_t*)\"{field[0]}\", {field[1]}, {field[2]}, (void*) {field[3]}, (void*) {field[4]}, {field[5]} }},")
    c_code.append("};")
    return c_code


def get_peripheral_handler(peripheral, handlers_init: list):
    """
    Returns a string reference to the peripheral handler or empty string based on the
    peripheral type. Updates the `handlers_init` list if the peripheral type is GPIO.

    :param peripheral: A dictionary containing details about the peripheral. Must
        include keys "type" and "peripheral". If "type" is "GPIO", "peripheral"
        must also contain nested keys "port" and "pin".
    :type peripheral: dict
    :param handlers_init: A list to which generated initialization structures
        for GPIO peripherals will be appended.
    :type handlers_init: list
    :return: A string reference to the peripheral handler or an empty string if
        the peripheral type is unsupported.
    :rtype: str
    """
    if peripheral["type"] == USART_DRIVER_NAME:
        return f"&huart{peripheral["peripheral"].removeprefix(USART_DRIVER_NAME)}"
    elif peripheral["type"] == GPIO_DRIVER_NAME:
        gpio_strict_name = f"GPIO_P{peripheral["peripheral"]["port"]}{peripheral["peripheral"]["pin"]}"
        handlers_init.extend(
            gen_struct_init("GPIO_ALLOC", gpio_strict_name,
                            [
                                ["gpio", f"GPIO{peripheral["peripheral"]["port"]}"],
                                ["pin", f"GPIO_PIN_{peripheral["peripheral"]["pin"]}"],
                            ],
                            True)
        )
        return f"&{gpio_strict_name}"
    elif peripheral["peripheral"] == "None":
        return "0"
    else:
        return ""


def gen_drivers_alloc(peri_config: dict, analysis: dict):
    """
    Generates driver allocation code and configuration table based on the given peripheral configuration.

    This function processes the provided peripheral configuration, extracts relevant details,
    and generates initialization code and a configuration table for driver allocation.
    It ensures proper handling of peripheral fields and their inclusion in the
    resulting generated code.

    :param analysis: Pre-analysis result
    :param peri_config: Dictionary representing peripheral configuration. Each peripheral
        is expected to have fields such as 'name', 'type', 'direction', and additional
        details required for proper driver initialization.
    :type peri_config: dict

    :return: List containing strings of generated initialization C code for the
        driver allocation process.
    :rtype: list
    """
    peri_list = []
    struct_init_c_code = []

    # Parse config
    for i, peripheral in enumerate(peri_config):
        # Check if the peripheral needs a buffer
        peri_buffer = "0"
        for buffer in analysis['buffers']:
            if peripheral['type'] == "USART" and peripheral["peripheral"] in buffer["name"]:
                peri_buffer = f"&{buffer['name']}"

        # Generate peripherals dictionary
        peri_fields = [
            peripheral["name"],
            peripheral["type"],
            peripheral["direction"],
            get_peripheral_handler(peripheral, struct_init_c_code),
            peri_buffer,
            i,
        ]
        peri_list.append(peri_fields)

    # Generate configuration table
    struct_init_c_code.extend(gen_table(DRIVER_ALLOC_TYPE, DRIVER_ALLOC_TABLE_NAME, peri_list, True))

    return struct_init_c_code


def gen_init_func(config: dict, analysis: dict):
    """
    Generates initialization code for drivers based on the provided configuration and analysis data.

    This function processes a dictionary configuration (`config`) that includes driver details,
    initialization sequences, and optional IT (Interrupt) enabled sequences for drivers. It also
    considers the analyzed `analysis` dictionary, which contains a list of drivers to initialize
    (`init_list`). The resulting initialization code is returned as a list of strings.

    :param config: Dictionary containing the drivers, their types, initialization sequences, and
        other configuration details.
    :type config: dict
    :param analysis: Dictionary containing details such as the list of drivers to initialize.
    :type analysis: dict
    :return: A list of strings forming the C code for the driver initialization function.
    :rtype: list
    """
    func_code = ["void drivers_init()"]
    func_code.extend("{")

    for driver_init in analysis['init_list']:
        # Get init sequence from config
        init_sequence = None
        init_sequence_it = None
        for seq in config['init_sequence']:
            if seq['driver'] == driver_init:
                init_sequence = seq['sequence']
                init_sequence_it = seq['it_enabled_sequence'] if 'it_enabled_sequence' in seq else None
                break

        for sequence in [init_sequence, init_sequence_it]:
            if sequence is not None:
                # Perform driver-specific actions
                if driver_init == USART_DRIVER_NAME:
                    for driver_to_init in config['drivers']:
                        if driver_to_init['type'] == USART_DRIVER_NAME:
                            func_code.append(f"    // {driver_to_init['peripheral']} initialization")
                            for init_call in sequence:
                                init_call = init_call.replace("<drv_name>", driver_to_init['peripheral'])
                                init_call = init_call.replace("<handler>", get_peripheral_handler(driver_to_init, []))
                                init_call = init_call.replace("<buffer>",
                                                              f"{driver_to_init['peripheral']}{BUFFER_NAME_SUFFIX}")
                                func_code.append(f"    {init_call}")
                            func_code.append("")

                else:
                    func_code.append(f"    // {driver_init} initialization")
                    for init_call in sequence:
                        func_code.append(f"    {init_call}")
                    func_code.append("")

    func_code.append("}")
    return func_code


def gen_handlers_func(config: dict):
    """
    Generates handler functions for interrupt-enabled drivers in the configuration.

    This function iterates through the configuration dictionary's list of drivers,
    and for each driver that has interrupt handling enabled, generates code for its
    corresponding interrupt handler function. It appends the generated function
    code to a list and returns it.

    :param config: Dictionary containing driver configuration. The dictionary should
        include a key 'drivers', where each driver item is a dictionary specifying
        details such as 'it_enabled' (whether the interrupt is enabled),
        'peripheral' (name of the peripheral), and 'type' (type of the driver).
    :type config: dict
    :return: List of strings, each representing lines of generated code for
        the interrupt handlers of enabled drivers.
    :rtype: list
    """
    func_code = []

    # For each driver with IT enabled
    for drv in config['drivers']:
        if 'it_enabled' in drv and drv['it_enabled']:
            func_code.append("")
            func_code.append(f"void {drv['peripheral']}_it_handler()")
            func_code.append("{")

            if drv['type'] == USART_DRIVER_NAME:
                func_code.append(f"    HAL_UART_IRQHandler({get_peripheral_handler(drv, [])});")
            func_code.append("}")

    return func_code


def gen_c_code(template: str, config: dict, analysis: dict, header: bool = False):
    """
    Generate source or header file based on a template and configuration parameters.

    This function processes a provided template file, replacing specific markers within
    the template with data from the configuration dictionary or analysis results. The
    output file may be generated as either a header or source file, depending on the
    value of the `header` parameter. The function ensures that appropriate includes,
    constants, and other data are added dynamically according to markers within the
    template.

    :param template: Path to the template file.
    :param config: Dictionary containing configuration data such as target file
                   information, include lists, and driver details.
    :param analysis: Dictionary with additional analysis data, like extra includes
                     to be added for source files.
    :param header: Boolean indicating whether to generate a header file (.h) or
                   source file (.c). Defaults to False for source file generation.
    :return: None
    """
    # Load template file
    template_lines = open(template).readlines()

    generated_lines = []

    file_ext = ".h" if header else ".c"

    # Replace markers by the new data
    for line in template_lines:
        markers = extract_marker(line)
        if markers is None:
            generated_lines.append(line)
        else:
            for marker in markers:
                if marker == FILENAME_MARKER:
                    generated_lines.append(line.replace(MARKER + marker, config['target_c_file']['name'] + file_ext))
                elif marker == DATE_MARKER:
                    generated_lines.append(line.replace(MARKER + marker, datetime.now().strftime("%d-%m-%Y")))
                elif marker == AUTHOR_MARKER:
                    generated_lines.append(
                        line.replace(MARKER + marker,
                                     "Auto-generated by " + os.path.splitext(os.path.basename(__file__))[0]))
                elif marker == INCLUDE_MARKER:
                    if header:
                        generated_lines.extend(gen_includes(config['includes_h']))
                    else:
                        includes_list = config['includes_c']
                        includes_list.extend(analysis['includes_c'])
                        generated_lines.extend(gen_includes(includes_list))
                elif marker == CONST_MARKER:
                    if header:
                        generated_lines.append(f"extern const {DRIVER_ALLOC_TYPE} {DRIVER_ALLOC_TABLE_NAME}[];")

                        # Generate buffers declaration
                        for buffer in analysis['buffers']:
                            generated_lines.append(f"extern RX_BUFFER {buffer['name']};")
                    else:
                        # Generate buffers declaration
                        for buffer in analysis['buffers']:
                            generated_lines.append(f"uint8_t {buffer['name']}_BUF[{buffer['size']}];")
                            generated_lines.extend(gen_struct_init("RX_BUFFER", buffer['name'],
                                                                   [
                                                                       ["buffer",
                                                                        f"{buffer['name']}_BUF"],
                                                                       ["size",
                                                                        "0"],
                                                                   ],
                                                                   False))

                        generated_lines.extend(gen_drivers_alloc(config['drivers'], analysis))

                elif marker == IFNDEF_MARKER:
                    generated_lines.append(
                        f"#ifndef {config['target_c_file']['name'].upper()}_{file_ext.removeprefix('.').upper()}")
                    generated_lines.append(
                        f"#define {config['target_c_file']['name'].upper()}_{file_ext.removeprefix('.').upper()}")
                elif marker == DEFINE_MARKER:
                    generated_lines.extend(gen_defines([[
                        "DRIVERS_ALLOC_SIZE",
                        str(len(config['drivers'])),
                    ]]))
                    for act in analysis['activations']:
                        generated_lines.append(f"#define {act}")
                    for buffer_size in analysis['buffers_size']:
                        generated_lines.append(f"#define {buffer_size} {analysis['buffers_size'][buffer_size]}")
                elif marker == FUNCTIONS_MARKER:
                    if header:
                        generated_lines.append("void drivers_init();")
                    else:
                        generated_lines.extend(gen_init_func(config, analysis))
                        generated_lines.extend(gen_handlers_func(config))
                else:
                    print(f"Unknown marker: {MARKER}{marker}")

    # Write the target file
    with open(os.path.join(config['target_c_file']['directory'], "Inc" if header else "Src",
                           config['target_c_file']['name'] + file_ext), 'w') as f:
        f.writelines([line + "\n" if not line.endswith("\n") else line for line in generated_lines])
    print(f"File {config['target_c_file']['name']}{file_ext} generated")


def gen_rust_code(config: dict):
    """
    Generates a Rust source file with interrupt bindings and handler functions for specified drivers.

    This function creates a Rust file containing interrupt handler bindings and
    functions based on the provided configuration. If a driver specifies the
    'enabled IT' (`it_enabled`), the function writes corresponding interrupt
    bindings and function definitions into the output file. The output file path
    and name are customized by the provided configuration.

    :param config: A dictionary containing the configuration for code generation.
        The expected keys include:
            - drivers: A list of dictionaries, where each dictionary represents a
              driver configuration. Each driver dictionary can include:
                  - it_enabled: A boolean indicating if the interrupt is enabled
                    for this driver.
                  - peripheral: A string denoting the peripheral's name.
            - target_rust_file: A dictionary specifying the target output file with
              keys:
                  - directory: The directory where the file will be saved.
                  - name: The name of the output file (without extension).
    :return: None
    """
    generated_lines = []

    file_ext = ".rs"

    generated_lines.append("use stm32f7::stm32f769::interrupt;")
    generated_lines.append("")
    generated_lines.append("unsafe extern \"C\" {")

    # Generate bindings
    bindings = []

    # For each driver
    for drv in config['drivers']:
        # If the driver has an IT enabled
        if 'it_enabled' in drv and drv['it_enabled']:
            binding = f"{drv['peripheral']}_it_handler"
            generated_lines.append(f"    pub fn {binding}();")
            bindings.append(binding)

    generated_lines.append("}")
    generated_lines.append("")

    # For each driver
    for drv in config['drivers']:
        # If the driver has an IT enabled
        if 'it_enabled' in drv and drv['it_enabled']:
            generated_lines.append("#[allow(non_snake_case)]")
            generated_lines.append("#[interrupt]")
            generated_lines.append(f"fn {drv['peripheral']}() {{")
            generated_lines.append(f"    unsafe {{ {drv['peripheral']}_it_handler(); }}")
            generated_lines.append("}")

    # Write the target file
    with open(os.path.join(config['target_rust_file']['directory'],
                           config['target_rust_file']['name'] + file_ext), 'w') as f:
        f.writelines([line + "\n" if not line.endswith("\n") else line for line in generated_lines])
    print(f"File {config['target_rust_file']['name']}{file_ext} generated")


#################
# Script begins #
#################
if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("Usage: python gen_drivers_alloc.py <input_file>")
        exit(1)

    # Load YAML configuration file
    input_file = sys.argv[1]
    gen_config = yaml.load(open(input_file), Loader=yaml.FullLoader)

    print("Generating drivers allocation...")

    # Pre-analysis
    pre_analysis = {'includes_c': [], 'activations': [], 'init_list': [], 'buffers': [], 'buffers_size': {}}

    for driver in gen_config['drivers']:
        # Add specific includes for drivers
        if driver['type'] == USART_DRIVER_NAME and "usart.h" not in pre_analysis['includes_c']:
            pre_analysis['includes_c'].append("usart.h")
        # Add activation for each driver
        activation = f"DRIVER_ACTIVATE_{driver['type']}"
        if activation not in pre_analysis['activations']:
            pre_analysis['activations'].append(activation)
        # Add driver init list
        if driver['type'] not in pre_analysis['init_list']:
            pre_analysis['init_list'].append(driver['type'])

            # Add init includes and buffers in the list
            for init in gen_config['init_sequence']:
                if init['driver'] == driver['type']:
                    for include in init['includes']:
                        if include not in pre_analysis['includes_c']:
                            pre_analysis['includes_c'].append(include)

        # Add driver buffer
        for init in gen_config['init_sequence']:
            if init['driver'] == driver['type'] and 'it_enabled' in driver and driver['it_enabled']:
                pre_analysis['buffers'].append(
                    {'name': driver['peripheral'] + BUFFER_NAME_SUFFIX, 'size': init['driver'] + BUFFER_SIZE_SUFFIX})
                if init['driver'] + BUFFER_SIZE_SUFFIX not in pre_analysis['buffers_size']:
                    pre_analysis['buffers_size'][init['driver'] + BUFFER_SIZE_SUFFIX] = init['buffer_size']

    # Generate C file
    gen_c_code(os.path.join(Path(__file__).resolve().parent, C_TEMPLATE_FILE), gen_config, pre_analysis)

    # Generate H file
    gen_c_code(os.path.join(Path(__file__).resolve().parent, H_TEMPLATE_FILE), gen_config, pre_analysis, True)

    # Generate Rust file
    gen_rust_code(gen_config)
    
    exit(0)
