package org.mamba.rattler;

import java.util.List;
import java.util.Optional;

public class CreateOpts {
    private final List<String> channels;
    private final List<String> specs;

    private final boolean dryRun;
    private final Optional<String> platform;
    private final List<String> virtualPackage;

    public CreateOpts(List<String> channels, List<String> specs, boolean dryRun, Optional<String> platform, List<String> virtualPackage) {
        this.channels = channels;
        this.specs = specs;
        this.dryRun = dryRun;
        this.platform = platform;
        this.virtualPackage = virtualPackage;
    }

    public List<String> getChannels() {
        return channels;
    }

    public List<String> getSpecs() {
        return specs;
    }

    public boolean isDryRun() {
        return dryRun;
    }

    public Optional<String> getPlatform() {
        return platform;
    }

    public List<String> getVirtualPackage() {
        return virtualPackage;
    }
}
