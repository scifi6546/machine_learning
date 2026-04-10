// machine_lerning.cpp : Defines the entry point for the application.
//

#include "machine_lerning.h"
#include <cstdio>
#include "lodepng.hpp"
using namespace std;

int main()
{
	unsigned int width = 1024;
	unsigned int height = 1024;
	unsigned int pixel_size = 4;
	unsigned char* pixel_data = (unsigned char*) malloc(width * height * pixel_size) ;
	for (unsigned int x = 0; x < width; x++) {
		for (unsigned int y = 0; y < height; y++) {
			pixel_data[4 * width * y + 4 * x + 0] = 255 * !(x & y);
			pixel_data[4 * width * y + 4 * x + 1] = x ^ y;
			pixel_data[4 * width * y + 4 * x + 2] = x | y;
			pixel_data[4 * width * y + 4 * x + 3] = 255;
		}
	}
	unsigned error = lodepng_encode32_file("./test.png", pixel_data, width, height);

	/*if there's an error, display it*/
	if (error) printf("error %u: %s\n", error, lodepng_error_text(error));
	cout << "Hello CMake." << endl;
	return 0;
}
