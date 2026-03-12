# SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
# SPDX-License-Identifier: GPL-3.0-or-later

import argparse
from PIL import Image, ImageDraw, ImageFont
import os

def ensure_dir(path):
    """Create the full path including all intermediate directories."""
    os.makedirs(os.path.dirname(path), exist_ok=True)

def process_image(input_file, output_filename, threshold):
    # Check if the input file exists
    if not os.path.exists(input_file):
        print(f"Error: Input file '{input_file}' not found.")
        print(f"Current working directory: {os.getcwd()}")
        print("Please make sure the file exists and you're running the script from the correct directory.")
        return

    # Load the image
    try:
        image = Image.open(input_file).convert('L')  # Convert to grayscale
    except Exception as e:
        print(f"Error opening the image file: {e}")
        return

    # Convert to 1-bit monochrome based on threshold
    mono_image = image.point(lambda p: 255 if p > threshold else 0).convert('1')

    # Save as raw format
    raw_output_filename = os.path.splitext(output_filename)[0] + ".raw"
    try:
        ensure_dir(raw_output_filename)
        with open(raw_output_filename, 'wb') as f:
            f.write(mono_image.tobytes())
        print(f"Raw binary data saved as: {raw_output_filename}")
    except Exception as e:
        print(f"Error saving the raw file: {e}")

def create_font_image(font_name, font_height, character_width, output_filename, threshold):
    # Verify the font file exists
    if not os.path.isfile(font_name):
        raise FileNotFoundError(f"Font file '{font_name}' not found")
    
    # Load the font
    font = ImageFont.truetype(font_name, font_height)

    # Calculate the offset to vertically center the text in each cell
    ascent, descent = font.getmetrics()
    character_height = ascent
    print(f"Ascent: {ascent}, Descent: {descent}")

    # Constants
    columns = 16
    start_char = ord(' ')
    end_char = ord('~') + 1
    char_count = end_char - start_char
    rows = (char_count + columns - 1) // columns  # Calculate the number of rows needed

    # Calculate the width and height of the image
    img_width = character_width * columns
    img_height = character_height * rows

    # Create a new image with white background
    image = Image.new('L', (img_width, img_height), 0)  # 'L' for 8-bit grayscale

    # Create a draw object
    draw = ImageDraw.Draw(image)

    # Render the characters
    for i in range(char_count):
        char = chr(start_char + i)
        x = (i % columns) * character_width
        y = (i // columns) * character_height - (descent - 1)  # Fixed baseline for all characters
        draw.text((x, y), char, font=font, fill=255)  # Fill with black (0)

    # Convert to 1-bit monochrome based on threshold
    mono_image = image.point(lambda p: 255 if p > threshold else 0).convert('1')

    # Save the image as PNG (optional, for visual inspection)
    png_output_filename = os.path.splitext(output_filename)[0] + ".png"
    ensure_dir(png_output_filename)
    mono_image.save(png_output_filename)
    print(f"Monochrome PNG saved as: {png_output_filename}")

    # Save as raw format
    raw_output_filename = os.path.splitext(output_filename)[0] + ".raw"
    ensure_dir(raw_output_filename)
    with open(raw_output_filename, 'wb') as f:
        f.write(mono_image.tobytes())
    print(f"Raw binary data saved as: {raw_output_filename}")

if __name__ == '__main__':
    parser = argparse.ArgumentParser(description="Generate an embedded font image or process a PNG, and save as raw binary")
    parser.add_argument('--input', '-i', required=True, help='Path to the input font file or PNG image')
    parser.add_argument('--output', '-o', required=True, help='Base name for output files (without extension)')
    parser.add_argument('--mode', '-m', choices=['font', 'image'], required=True, help='Processing mode: "font" or "image"')
    parser.add_argument('--height', '-H', type=int, help='Font height in pixels (required for font mode)')
    parser.add_argument('--width', '-w', type=int, help='Width of each character (required for font mode)')
    parser.add_argument('--threshold', '-t', type=int, default=128, help='Threshold for monochrome conversion (0-255, default: 128)')
    
    args = parser.parse_args()
    print(args)

    if args.mode == 'font':
        if args.height is None or args.width is None:
            parser.error("Both --height and --width are required when mode is 'font'")
        create_font_image(args.input, args.height, args.width, args.output, args.threshold)
    else:  # image mode
        process_image(args.input, args.output, args.threshold)
