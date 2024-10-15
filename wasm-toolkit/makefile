# Define variables for the file paths
PKG_DIR = pkg
NODE_JS = $(PKG_DIR)/defuse_wasm_toolkit_node.js
BUNDLER_JS = $(PKG_DIR)/defuse_wasm_toolkit_bundler.js
PACKAGE_JSON = $(PKG_DIR)/package.json

# Define the default target
all: build_node build_bundler update_package_json

# Build for nodejs target and rename the output file
build_node:
	@echo "Building for nodejs target..."
	wasm-pack build --release --target nodejs
	mv $(PKG_DIR)/defuse_wasm_toolkit.js $(NODE_JS)

# Build for bundler target and rename the output file
build_bundler:
	@echo "Building for bundler target..."
	wasm-pack build --release --target bundler
	mv $(PKG_DIR)/defuse_wasm_toolkit.js $(BUNDLER_JS)

# Update the package.json to include the new files and update fields
update_package_json:
	@echo "Updating package.json..."

	# Use jq to add or remove items in the JSON fields, including "./" in sideEffects paths
	jq 'del(.files[] | select(. == "defuse_wasm_toolkit.js")) | \
	    .files += ["defuse_wasm_toolkit_node.js", "defuse_wasm_toolkit_bundler.js"] | \
	    .main = "defuse_wasm_toolkit_node.js" | \
	    .browser = "defuse_wasm_toolkit_bundler.js" | \
	    del(.sideEffects[] | select(. == "./defuse_wasm_toolkit.js")) | \
	    .sideEffects += ["./defuse_wasm_toolkit_node.js", "./defuse_wasm_toolkit_bundler.js"]' \
	    $(PACKAGE_JSON) > $(PACKAGE_JSON).tmp && mv $(PACKAGE_JSON).tmp $(PACKAGE_JSON)

	@echo "Build and package update completed!"

# Clean target to remove old builds if needed
clean:
	@echo "Cleaning old builds..."
	rm -rf $(PKG_DIR)

.PHONY: all build_node build_bundler update_package_json clean