# CMake generated Testfile for 
# Source directory: /app/src/starkware/starknet/std_contracts/ERC20
# Build directory: /app/build/Release/src/starkware/starknet/std_contracts/ERC20
# 
# This file includes the relevant testing commands required for 
# testing this directory and lists subdirectories to be tested as well.
add_test([=[cairo_erc20_test]=] "/app/build/Release/src/starkware/starknet/std_contracts/ERC20/cairo_erc20_test")
set_tests_properties([=[cairo_erc20_test]=] PROPERTIES  _BACKTRACE_TRIPLES "/app/src/cmake_utils/python_rules.cmake;247;add_test;/app/src/cmake_utils/python_rules.cmake;298;python_test;/app/src/starkware/starknet/std_contracts/ERC20/CMakeLists.txt;23;full_python_test;/app/src/starkware/starknet/std_contracts/ERC20/CMakeLists.txt;0;")
