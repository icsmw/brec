package com.icsmw.brec;

public final class ClientBindings {
    static {
        System.loadLibrary("bindings");
    }

    private ClientBindings() {}

    public static native Object decodePacket(byte[] bytes);

    public static native byte[] encodePacket(Object packet);
}
