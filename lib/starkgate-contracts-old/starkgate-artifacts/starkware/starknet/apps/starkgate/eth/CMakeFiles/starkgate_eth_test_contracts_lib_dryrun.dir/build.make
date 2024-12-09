# CMAKE generated file: DO NOT EDIT!
# Generated by "Unix Makefiles" Generator, CMake Version 3.22

# Delete rule output on recipe failure.
.DELETE_ON_ERROR:

#=============================================================================
# Special targets provided by cmake.

# Disable implicit rules so canonical targets will work.
.SUFFIXES:

# Disable VCS-based implicit rules.
% : %,v

# Disable VCS-based implicit rules.
% : RCS/%

# Disable VCS-based implicit rules.
% : RCS/%,v

# Disable VCS-based implicit rules.
% : SCCS/s.%

# Disable VCS-based implicit rules.
% : s.%

.SUFFIXES: .hpux_make_needs_suffix_list

# Command-line flag to silence nested $(MAKE).
$(VERBOSE)MAKESILENT = -s

#Suppress display of executed commands.
$(VERBOSE).SILENT:

# A target that is always out of date.
cmake_force:
.PHONY : cmake_force

#=============================================================================
# Set environment variables for the build.

# The shell in which to execute make rules.
SHELL = /bin/sh

# The CMake executable.
CMAKE_COMMAND = /usr/local/lib/python3.7/site-packages/cmake/data/bin/cmake

# The command to remove a file.
RM = /usr/local/lib/python3.7/site-packages/cmake/data/bin/cmake -E rm -f

# Escaping for special characters.
EQUALS = =

# The top-level source directory on which CMake was run.
CMAKE_SOURCE_DIR = /app

# The top-level build directory on which CMake was run.
CMAKE_BINARY_DIR = /app/build/Release

# Utility rule file for starkgate_eth_test_contracts_lib_dryrun.

# Include any custom commands dependencies for this target.
include src/starkware/starknet/apps/starkgate/eth/CMakeFiles/starkgate_eth_test_contracts_lib_dryrun.dir/compiler_depend.make

# Include the progress variables for this target.
include src/starkware/starknet/apps/starkgate/eth/CMakeFiles/starkgate_eth_test_contracts_lib_dryrun.dir/progress.make

src/starkware/starknet/apps/starkgate/eth/CMakeFiles/starkgate_eth_test_contracts_lib_dryrun: python_libs/starkgate_eth_test_contracts_lib.info.dryrun

python_libs/starkgate_eth_test_contracts_lib.info.dryrun: ../../src/cmake_utils/gen_py_lib.py
	@$(CMAKE_COMMAND) -E cmake_echo_color --switch=$(COLOR) --blue --bold --progress-dir=/app/build/Release/CMakeFiles --progress-num=$(CMAKE_PROGRESS_1) "Generating ../../../../../../python_libs/starkgate_eth_test_contracts_lib.info.dryrun"
	cd /app/build/Release/src/starkware/starknet/apps/starkgate/eth && ../../../../../../../../src/cmake_utils/gen_py_lib.py --name starkgate_eth_test_contracts_lib --lib_dir /app/build/Release/src/starkware/starknet/apps/starkgate/eth/starkgate_eth_test_contracts_lib --files starkware/starknet/apps/starkgate/eth/test_contracts.py starkware/starknet/apps/starkgate/eth/StarknetERC20BridgeTester.json starkware/starknet/apps/starkgate/eth/StarknetEthBridgeTester.json --lib_deps starkware_contracts_utils_lib --py_exe_deps --cmake_dir src/starkware/starknet/apps/starkgate/eth --prefix starkware/starknet/apps/starkgate/eth/ --output /app/build/Release/python_libs/starkgate_eth_test_contracts_lib.info.dryrun

starkgate_eth_test_contracts_lib_dryrun: python_libs/starkgate_eth_test_contracts_lib.info.dryrun
starkgate_eth_test_contracts_lib_dryrun: src/starkware/starknet/apps/starkgate/eth/CMakeFiles/starkgate_eth_test_contracts_lib_dryrun
starkgate_eth_test_contracts_lib_dryrun: src/starkware/starknet/apps/starkgate/eth/CMakeFiles/starkgate_eth_test_contracts_lib_dryrun.dir/build.make
.PHONY : starkgate_eth_test_contracts_lib_dryrun

# Rule to build all files generated by this target.
src/starkware/starknet/apps/starkgate/eth/CMakeFiles/starkgate_eth_test_contracts_lib_dryrun.dir/build: starkgate_eth_test_contracts_lib_dryrun
.PHONY : src/starkware/starknet/apps/starkgate/eth/CMakeFiles/starkgate_eth_test_contracts_lib_dryrun.dir/build

src/starkware/starknet/apps/starkgate/eth/CMakeFiles/starkgate_eth_test_contracts_lib_dryrun.dir/clean:
	cd /app/build/Release/src/starkware/starknet/apps/starkgate/eth && $(CMAKE_COMMAND) -P CMakeFiles/starkgate_eth_test_contracts_lib_dryrun.dir/cmake_clean.cmake
.PHONY : src/starkware/starknet/apps/starkgate/eth/CMakeFiles/starkgate_eth_test_contracts_lib_dryrun.dir/clean

src/starkware/starknet/apps/starkgate/eth/CMakeFiles/starkgate_eth_test_contracts_lib_dryrun.dir/depend:
	cd /app/build/Release && $(CMAKE_COMMAND) -E cmake_depends "Unix Makefiles" /app /app/src/starkware/starknet/apps/starkgate/eth /app/build/Release /app/build/Release/src/starkware/starknet/apps/starkgate/eth /app/build/Release/src/starkware/starknet/apps/starkgate/eth/CMakeFiles/starkgate_eth_test_contracts_lib_dryrun.dir/DependInfo.cmake --color=$(COLOR)
.PHONY : src/starkware/starknet/apps/starkgate/eth/CMakeFiles/starkgate_eth_test_contracts_lib_dryrun.dir/depend

