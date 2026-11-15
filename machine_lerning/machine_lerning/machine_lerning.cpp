// machine_lerning.cpp : Defines the entry point for the application.
//

#include "machine_lerning.h"
#include <cstdio>
#include <math.h>

#include "lodepng.hpp"
static unsigned int ctable [1024][4];
#define NUMBER_SCALOGRAMS 9
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
	const char* paths[NUMBER_SCALOGRAMS] = {
		"ak018fcnsk91_GHO_BHE",
		"ak018fcnsk91_GHO_BHN",
		"ak018fcnsk91_GHO_BHZ",
		"ak018fcnsk91_RC01_BHE",
		"ak018fcnsk91_RC01_BHN",
		"ak018fcnsk91_RC01_BHZ",
		"ak018fcnsk91_SWD_BHE",
		"ak018fcnsk91_SWD_BHN",
		"ak018fcnsk91_SWD_BHZ",

	};
	for (int i = 0; i < NUMBER_SCALOGRAMS; i++) {
		printf("%s\n", paths[i]);
		const char metadata_path[2000] = "";
		sprintf((char*)metadata_path, "../../../../load_data/metadata/%s_metadata.txt", paths[i]);
		const char spectrogram_path[2000] = "";
		sprintf((char*)spectrogram_path, "../../../../load_data/spectrogram/%s_spectrogram.bin", paths[i]);
		printf("%s\n", spectrogram_path);

		ScalogramMetadata metadata = load_metadata(metadata_path);
		printf("width: %i height: %i\n", metadata.width, metadata.height);

		FILE* scalogram = fopen(spectrogram_path, "rb");
		if (scalogram == NULL) {
			printf("error reading scalogram\n");
		}

		size_t scalogram_element_count = metadata.width * metadata.height;
		size_t scalogram_buffer_size = scalogram_element_count * sizeof(double);
		double* scalogram_data = (double*)calloc(scalogram_element_count, sizeof(double));

		size_t elements_read = fread_s((void*)scalogram_data, scalogram_buffer_size, sizeof(double), scalogram_element_count, scalogram);
		if (elements_read != scalogram_element_count) {
			printf("scalogram not fully \n");
		}
		double max = 0.;
		for (size_t i = 0; i < scalogram_element_count; i++) {
			double val = log(scalogram_data[i]);
			if (val > max) {
				max = val;
			}
		}
		max = 5.0;
		printf("max value: %f\n", max);

		unsigned int pixel_size = 4;
		int i1, j, R, G, B, A = 253;

		// Create Pixel index table
		for (i1 = 0, R = 0, G = 0, B = 255; i1 < 512; i1 += 2) {
			j = i1 + 1;
			ctable[i1][0] = R;
			ctable[i1][1] = G;
			ctable[i1][2] = B;
			ctable[i1][3] = A;
			if (G < 255)
				G++;
			ctable[j][0] = R;
			ctable[j][1] = G;
			ctable[j][2] = B;
			ctable[j][3] = A;
			B--;
		}
		for (i1 = 512, R = 0, G = 255, B = 0; i1 < 1024; i1 += 2) {
			j = i1 + 1;
			ctable[i1][0] = R;
			ctable[i1][1] = G;
			ctable[i1][2] = B;
			ctable[i1][3] = A;
			if (R < 255)
				R++;	
			ctable[j][0] = R;
			ctable[j][1] = G;
			ctable[j][2] = B;
			ctable[j][3] = A;
			G--;
		}
		// Display pixel table
		int table_width = 200;
		int table_height = 1024;
		unsigned char* table_data = (unsigned char*)malloc(table_width * table_height * pixel_size);
		for (j = 0; j < table_height; j++) {
			int pixIndex = j;
			unsigned char pixelR = ctable[pixIndex][0];
			unsigned char pixelG = ctable[pixIndex][1];
			unsigned char pixelB = ctable[pixIndex][2];
			unsigned char pixelA = ctable[pixIndex][3];
			for (i1 = 0; i1 < table_width; i1++) {
				table_data[4 * table_width * j + 4 * i1 + 0] = pixelR;
				table_data[4 * table_width * j + 4 * i1 + 1] = pixelG;
				table_data[4 * table_width * j + 4 * i1 + 2] = pixelB;
				table_data[4 * table_width * j + 4 * i1 + 3] = pixelA;
			}
		}
		{
			const char output_table[2000] = "";
			sprintf((char*)output_table, "../../../../load_data/saved_images/color_table.png", ctable);
			unsigned error = lodepng_encode32_file(output_table, table_data, table_width, table_height);
			if (error) printf("error intable: %d\n", error, lodepng_error_text(error));
		}

		unsigned char* pixel_data = (unsigned char*)malloc(metadata.width * metadata.height * pixel_size);
		for (unsigned int x = 0; x < metadata.width; x++) {
			for (unsigned int y = 0; y < metadata.height; y++) {
				double value = log(scalogram_data[x * metadata.height + y]) / max;
				double pixel_floating = value * 1023.;
				if (pixel_floating > 1023.) {
					pixel_floating = 1023.;
				}
				int pixIndex = (int)(pixel_floating);
				unsigned char pixelR = ctable[pixIndex][0];
				unsigned char pixelG = ctable[pixIndex][1];
				unsigned char pixelB = ctable[pixIndex][2];
				unsigned char pixelA = ctable[pixIndex][3];
				
				pixel_data[4 * metadata.width * y + 4 * x + 0] = pixelR;
				pixel_data[4 * metadata.width * y + 4 * x + 1] = pixelG;
				pixel_data[4 * metadata.width * y + 4 * x + 2] = pixelB;
				pixel_data[4 * metadata.width * y + 4 * x + 3] = pixelA;
			}
		}

/*	0ld
		for (unsigned int x = 0; x < metadata.width; x++) {
			for (unsigned int y = 0; y < metadata.height; y++) {
				double value = log(scalogram_data[x * metadata.height + y]) / max;
				double pixel_floating = value * 255.;
				if (pixel_floating > 255.) {
					pixel_floating = 255.;
				}
				unsigned char pixel_brightness = (unsigned char)(pixel_floating);
				pixel_data[4 * metadata.width * y + 4 * x + 0] = pixel_brightness;
				pixel_data[4 * metadata.width * y + 4 * x + 1] = pixel_brightness;
				pixel_data[4 * metadata.width * y + 4 * x + 2] = pixel_brightness;
				pixel_data[4 * metadata.width * y + 4 * x + 3] = pixel_brightness;
			}
		}
*/
		{
			const char output_path[2000] = "";
			sprintf((char*)output_path, "../../../../load_data/saved_images/scalogram_%s.png", paths[i]);
			unsigned error = lodepng_encode32_file(output_path, pixel_data, metadata.width, metadata.height);
			if (error) printf("error %u: %s\n", error, lodepng_error_text(error));
		}

	}
	
	return 0;
}
