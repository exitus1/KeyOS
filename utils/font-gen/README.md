# Font and Image Processor

This Python script processes font files and images, converting them to raw binary format for embedded systems use. It can handle two modes of operation: font processing and image processing.

It is used to generate the font and icon data for the KeyOS bootloader.

## Requirements

- Python 3.x
- Pillow library (`pip install Pillow`)

## Usage

The script can be run from the command line with various arguments:

```
python font-gen.py --mode <mode> --input <input_file> --output <output_name> [--width <width>] [--height <height>] [--threshold <threshold>]
```

### Arguments

- `--mode`, `-m`: Processing mode, either 'font' or 'image' (required)
- `--input`, `-i`: Path to the input font file or PNG image (required)
- `--output`, `-o`: Base name for output files (without extension) (required)
- `--width`, `-w`: Width of each character (required for font mode, not used in image mode)
- `--height`, `-H`: Font height in pixels (required for font mode, not used in image mode)
- `--threshold`, `-t`: Threshold for monochrome conversion (0-255, default: 128)

## Examples

### Font Processing Mode

To process a font file (e.g., SourceCodePro-Medium.ttf) with a height of 24 pixels and width of 14 pixels:

```
python font-gen.py --mode font --input fonts/SourceCodePro-Medium.ttf --height 24 --width 14 --output out/source-code-pro-24 --threshold 128
```

This command will generate two files:

- `out/source-code-pro-24.png`: A PNG image of the processed font (for visual inspection)
- `out/source-code-pro-24.raw`: The raw binary data of the processed font

### Image Processing Mode

To process an image file (e.g., bootloader-icons.png):

```
python font-gen.py --mode image --input icons/bootloader-icons.png --output out/bootloader-icons --threshold 128
```

This command will generate one file:

- `out/bootloader-icons.raw`: The raw binary data of the processed image

Note: For image mode, the width is automatically determined from the input image.

## Output

The script generates raw binary files (.raw) in both modes. In font mode, it also generates a PNG file for visual inspection.

The raw files contain the image data in a 1-bit per pixel format, where each bit represents a pixel (0 for black, 1 for white).
