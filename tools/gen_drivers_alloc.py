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

DRIVER_ALLOC_TYPE = "DRIVER_ALLOC"
DRIVER_ALLOC_TABLE_NAME = "DRIVERS_ALLOC"


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
    const = "const" if is_const else ""

    c_code.append(f"{const} {struct_type} {struct_name} = {{")
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
        c_code.append(f"    {{ (uint8_t*)\"{field[0]}\", {field[1]}, {field[2]}, (void*) {field[3]}, {field[4]} }},")
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
    if peripheral["type"] == "USART":
        return f"&huart{peripheral["peripheral"].removeprefix('USART')}"
    elif peripheral["type"] == "GPIO":
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


def gen_drivers_alloc(peri_config: dict):
    """
    Generates driver allocation code and configuration table based on the given peripheral configuration.

    This function processes the provided peripheral configuration, extracts relevant details,
    and generates initialization code and a configuration table for driver allocation.
    It ensures proper handling of peripheral fields and their inclusion in the
    resulting generated code.

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
        peri_fields = [
            peripheral["name"],
            peripheral["type"],
            peripheral["direction"],
            get_peripheral_handler(peripheral, struct_init_c_code),
            i,
        ]
        peri_list.append(peri_fields)

    # Generate configuration table
    struct_init_c_code.extend(gen_table(DRIVER_ALLOC_TYPE, DRIVER_ALLOC_TABLE_NAME, peri_list, True))

    return struct_init_c_code


def gen_code(template: str, config: dict, header: bool = False):
    """
    Generate code using a template file and a configuration dictionary. This function
    reads the provided template file, processes specific markers within the template,
    and outputs a generated file based on the configuration and the provided header
    flag. This is useful for automating the creation of source or header files in C
    or other programming environments.

    :param template: Path to the template file
    :type template: str
    :param config: Configuration dictionary containing various attributes needed
                   for generation such as target file name, directory, includes,
                   drivers, etc.
    :type config: dict
    :param header: Flag to specify whether to generate a header (.h) file or a
                   source (.c) file. Defaults to False, indicating source file
                   generation.
    :type header: bool
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
                    generated_lines.append(line.replace(MARKER + marker, config['target_file']['name'] + file_ext))
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
                        generated_lines.extend(gen_includes(config['includes_c']))
                elif marker == CONST_MARKER:
                    if header:
                        generated_lines.append(f"extern const {DRIVER_ALLOC_TYPE} {DRIVER_ALLOC_TABLE_NAME}[];")
                    else:
                        generated_lines.extend(gen_drivers_alloc(config['drivers']))
                elif marker == IFNDEF_MARKER:
                    generated_lines.append(
                        f"#ifndef {config['target_file']['name'].upper()}_{file_ext.removeprefix('.').upper()}")
                    generated_lines.append(
                        f"#define {config['target_file']['name'].upper()}_{file_ext.removeprefix('.').upper()}")
                elif marker == DEFINE_MARKER:
                    generated_lines.extend(gen_defines([[
                        "DRIVERS_ALLOC_SIZE",
                        str(len(config['drivers'])),
                    ]]))
                else:
                    print(f"Unknown marker: {MARKER}{marker}")

    # Write the target file
    with open(os.path.join(config['target_file']['directory'], "Inc" if header else "Src",
                           config['target_file']['name'] + file_ext), 'w') as f:
        f.writelines([line + "\n" if not line.endswith("\n") else line for line in generated_lines])
    print(f"File {config['target_file']['name']}{file_ext} generated")


#################
# Script begins #
#################
if len(sys.argv) != 2:
    print("Usage: python gen_drivers_alloc.py <input_file>")
    exit(1)

# Load YAML configuration file
input_file = sys.argv[1]
gen_config = yaml.load(open(input_file), Loader=yaml.FullLoader)

print("Generating drivers allocation...")

# Generate C file
gen_code(os.path.join(Path(__file__).resolve().parent, C_TEMPLATE_FILE), gen_config)

# Generate H file
gen_code(os.path.join(Path(__file__).resolve().parent, H_TEMPLATE_FILE), gen_config, True)
