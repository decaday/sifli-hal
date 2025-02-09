"""
Convert C array pinmux data to pinmux.yaml for sifli-hal

input format:
(copied from https://github.com/OpenSiFli/SiFli-SDK/blob/main/drivers/cmsis/sf32lb52x/bf0_pin_const.c)

const unsigned short pin_pad_func_hcpu[][16] =
{
    ....
    {GPIO_A0,   LCDC1_SPI_RSTB, 0,  0,  PA00_I2C_UART,  PA00_TIM,   0,  LCDC1_8080_RSTB,    0,  0,  0,  0,  0,  0,  0,  0},
    {GPIO_A1,   0,  0,  0,  PA01_I2C_UART,  PA01_TIM,   0,  0,  0,  0,  0,  0,  0,  0,  0,  0},
    {GPIO_A2,   LCDC1_SPI_TE,   0,  I2S1_MCLK,  PA02_I2C_UART,  PA02_TIM,   LCDC1_JDI_B2,   LCDC1_8080_TE,  0,  DBG_DO0,    0,  EDT_CHANNEL_IN0,    0,  0,  0,  0},
    ....
};

output:  output\pinmux.yaml
After confirmation, it will be copied to:
sifli-hal\data\sf32lb52x\pinmux.yaml

"""

import re
import yaml
import os

def parse_c_array(input_text):
    """
    Parse C array input text and extract pin configurations
    Returns a list of dictionaries containing pin data
    """
    # Initialize result list
    result = []
    
    # Regular expression to match array rows
    row_pattern = r'{([^}]+)}'
    
    # Find all rows in the input text
    rows = re.findall(row_pattern, input_text)
    
    for row in rows:
        # Split the row into items and clean them
        items = [item.strip() for item in row.split(',')]
        
        # Skip if first item doesn't match GPIO_A\d pattern
        if not re.match(r'GPIO_A\d+', items[0]):
            continue
            
        # Create pin entry
        pin_entry = {
            'pin': items[0],
            'functions': []
        }
        
        # Add functions with their values
        for value, func in enumerate(items):
            if func != '0' and func.strip():
                pin_entry['functions'].append({
                    'function': func,
                    'value': value
                })
        
        result.append(pin_entry)
    
    return result

def create_yaml(pin_data):
    """
    Create YAML structure from pin data
    """
    return {'HCPU': pin_data}

def save_yaml(data, output_path):
    """
    Save data to YAML file
    """
    # Create output directory if it doesn't exist
    os.makedirs(os.path.dirname(output_path), exist_ok=True)
    
    # Custom YAML dumper to avoid aliases
    class NoAliasDumper(yaml.SafeDumper):
        def ignore_aliases(self, data):
            return True
    
    # Write YAML file
    with open(output_path, 'w') as f:
        yaml.dump(data, f, default_flow_style=False, sort_keys=False, Dumper=NoAliasDumper)

def main():
    """
    Main function to process input and generate YAML
    """
    print("(Please paste C array data, then press Enter twice to finish)")
    
    input_lines = []
    while True:
        line = input()
        if line.strip() == "":
            break
        input_lines.append(line)
    
    # Combine input lines
    input_text = '\n'.join(input_lines)
    
    # Process the data
    pin_data = parse_c_array(input_text)
    yaml_data = create_yaml(pin_data)
    
    # Save to file
    output_path = 'output/pinmux.yaml'
    save_yaml(yaml_data, output_path)
    print(f"\nYAML file has been created at: {output_path}")

if __name__ == "__main__":
    main()