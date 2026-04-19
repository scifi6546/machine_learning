// machine_lerning.cpp : Defines the entry point for the application.
//

#include "machine_lerning.h"
#include <cstdio>
#include <math.h>
#include "lodepng.hpp"
using namespace std;

int main()
{
	FILE* scalogram = fopen("../../../../scalogram.bin","rb");
	if (scalogram == NULL) {
		printf("error reading scalogram\n");
	}

	unsigned int scalogram_width = 2939;
	unsigned int scalogram_height = 1024;
	size_t scalogram_element_count = scalogram_width * scalogram_height;
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
	unsigned int width = scalogram_width;
	unsigned int height = scalogram_height;
	unsigned int pixel_size = 4;
	unsigned char* pixel_data = (unsigned char*) malloc(width * height * pixel_size) ;
	for (unsigned int x = 0; x < width; x++) {
		for (unsigned int y = 0; y < height; y++) {
			double value = log(scalogram_data[y * width + x]) / max;
			unsigned char pixel_brightness = (unsigned char) (value * 255.);
			pixel_data[4 * width * y + 4 * x + 0] = pixel_brightness;
			pixel_data[4 * width * y + 4 * x + 1] = pixel_brightness;
			pixel_data[4 * width * y + 4 * x + 2] = pixel_brightness;
			pixel_data[4 * width * y + 4 * x + 3] = pixel_brightness;
		}
	}
	{
		unsigned error = lodepng_encode32_file("../../../../scalogram.png", pixel_data, width, height);
		if (error) printf("error %u: %s\n", error, lodepng_error_text(error));
	}
	{
		unsigned error = lodepng_encode32_file("../../../../test.png", pixel_data, width, height);

		/*if there's an error, display it*/
		if (error) printf("error %u: %s\n", error, lodepng_error_text(error));
	}

	

	cout << "Hello CMake." << endl;
	return 0;
}
