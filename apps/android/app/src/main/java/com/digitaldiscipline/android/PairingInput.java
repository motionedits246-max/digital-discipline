package com.digitaldiscipline.android;

import java.util.regex.Pattern;

final class PairingInput {
    private static final Pattern CODE = Pattern.compile("[A-Za-z0-9_-]{43}");
    private static final Pattern PIN = Pattern.compile("[0-9]{6,12}");

    static String normalize(String value) {
        StringBuilder code = new StringBuilder(value.length());
        for (int i = 0; i < value.length(); i++) {
            char character = value.charAt(i);
            if (Character.isWhitespace(character) || Character.isSpaceChar(character)
                || character == '\u200B' || character == '\uFEFF') continue;
            code.append(character);
        }
        return code.toString();
    }

    static boolean isCode(String value) {
        return CODE.matcher(value).matches();
    }

    static boolean isPin(String value) {
        return PIN.matcher(value).matches();
    }

    private PairingInput() {}
}
