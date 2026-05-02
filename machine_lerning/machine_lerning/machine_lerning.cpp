// machine_lerning.cpp : Defines the entry point for the application.
//

#include "machine_lerning.h"
#include <cstdio>
#include <math.h>
#include "lodepng.hpp"
using namespace std;
typedef struct ScalogramMetadata {
	unsigned int width;
	unsigned int height;
};
ScalogramMetadata load_metadata(const char* file_path) {
	FILE* metadata_file = fopen(file_path, "r");
	int in_width = 1;
	char character;
	ScalogramMetadata meta = {};
	int multiplier = 1;
	while ((character = fgetc(metadata_file)) != EOF) {
		if (in_width == 1) {
			if (character != ' ') {
				int number = ((int)character) - 0x30;
				meta.width *= 10;
				meta.width += number;
			}
			else {
				in_width = 0;
			}
		}
		else {
			int number = ((int)character) - 0x30;
			meta.height *= 10;
			meta.height += number;
		}
	}

	return meta;
}
int main()
{
	const char* metadata_path = "../../../../metadata.txt";
	const char* scalogram_path = "../../../../scalogram.bin";
	ScalogramMetadata metadata = load_metadata(metadata_path);
	printf("width: %i height: %i\n",metadata.width,metadata.height);
	FILE* scalogram = fopen(scalogram_path,"rb");
	if (scalogram == NULL) {
		printf("error reading scalogram\n");
	}


	unsigned int scalogram_height = 1024;
	size_t scalogram_element_count = metadata.width * scalogram_height;
	size_t scalogram_buffer_size = scalogram_element_count * sizeof(double);
	double* scalogram_data = (double*) calloc(scalogram_element_count, sizeof(double));

	size_t elements_read = fread_s((void *) scalogram_data, scalogram_buffer_size, sizeof(double), scalogram_element_count, scalogram);
	if (elements_read != scalogram_element_count) {
		printf("scalogram not fully read");
	}
	double max = 0.;
	for (size_t i = 0; i < scalogram_element_count; i++) {
		double val = log(scalogram_data[i]);
		if (val > max) {
			max = val;
		}
	}
	printf("max value: %f",max);
	
	unsigned int pixel_size = 4;
	unsigned char* pixel_data = (unsigned char*) malloc(metadata.width * metadata.height * pixel_size) ;
	for (unsigned int x = 0; x < metadata.width; x++) {
		for (unsigned int y = 0; y < metadata.height; y++) {
			double value = log(scalogram_data[y * metadata.width + x]) / max;
			unsigned char pixel_brightness = (unsigned char) (value * 255.);
			pixel_data[4 * metadata.width * y + 4 * x + 0] = pixel_brightness;
			pixel_data[4 * metadata.width * y + 4 * x + 1] = pixel_brightness;
			pixel_data[4 * metadata.width * y + 4 * x + 2] = pixel_brightness;
			pixel_data[4 * metadata.width * y + 4 * x + 3] = pixel_brightness;
		}
	}
	{
		unsigned error = lodepng_encode32_file("../../../../scalogram.png", pixel_data, metadata.width, metadata.height);
		if (error) printf("error %u: %s\n", error, lodepng_error_text(error));
	}
	{
		unsigned error = lodepng_encode32_file("../../../../test.png", pixel_data, metadata.width, metadata.height);

		/*if there's an error, display it*/
		if (error) printf("error %u: %s\n", error, lodepng_error_text(error));
	}

	

	cout << "Hello CMake." << endl;
	return 0;
}
