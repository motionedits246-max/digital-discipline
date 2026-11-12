package app.digitaldiscipline.android;

import android.util.Base64;
import org.json.JSONObject;

import java.io.DataInputStream;
import java.io.DataOutputStream;
import java.io.IOException;
import java.net.Inet4Address;
import java.net.InetAddress;
import java.net.InetSocketAddress;
import java.net.NetworkInterface;
import java.net.ServerSocket;
import java.net.Socket;
import java.nio.charset.StandardCharsets;
import java.security.GeneralSecurityException;
import java.security.SecureRandom;
import java.util.Arrays;
import java.util.Collections;
import javax.crypto.Cipher;
import javax.crypto.spec.GCMParameterSpec;
import javax.crypto.spec.SecretKeySpec;

final class FamilyLink implements AutoCloseable {
    private static final int PORT = 47163;
    private static final int MAX_REQUEST = 2048;
    private static final int MAX_RESPONSE = 1_048_576;
    private static final byte[] PROTOCOL = {'D', 'D', 'A', 1};
    private static final SecureRandom RANDOM = new SecureRandom();
    private final DeviceState state;
    private final ServerSocket listener;
    private final InetAddress address;

    FamilyLink(DeviceState state, InetAddress address) throws IOException {
        this.state = state;
        this.address = address;
        listener = new ServerSocket();
        try {
            listener.setReuseAddress(true);
            listener.bind(new InetSocketAddress(address, PORT));
        } catch (IOException e) {
            listener.close();
            throw e;
        }
        Thread worker = new Thread(this::serve, "discipline-family");
        worker.setDaemon(true);
        worker.start();
    }

    String code() { return "DD1-" + address.getHostAddress() + "-" + state.childKey(); }
    boolean onAddress(InetAddress current) { return address.equals(current) && !listener.isClosed(); }

    static InetAddress localAddress() {
        try {
            for (NetworkInterface net : Collections.list(NetworkInterface.getNetworkInterfaces())) {
                String name = net.getName();
                if (!net.isUp() || !name.matches("(wlan|swlan|ap|eth|softap|br_ap).*")) continue;
                for (InetAddress ip : Collections.list(net.getInetAddresses())) {
                    if (ip instanceof Inet4Address && ip.isSiteLocalAddress()) return ip;
                }
            }
        } catch (IOException ignored) {
            // The network interface may disappear while Wi-Fi changes.
        }
        return null;
    }

    static byte[] random(int size) {
        byte[] bytes = new byte[size];
        RANDOM.nextBytes(bytes);
        return bytes;
    }

    static String encode(byte[] bytes) {
        return Base64.encodeToString(bytes, Base64.URL_SAFE | Base64.NO_WRAP | Base64.NO_PADDING);
    }

    static byte[] decode(String text) { return Base64.decode(text, Base64.URL_SAFE | Base64.NO_WRAP); }

    static JSONObject request(String code, JSONObject message) throws Exception {
        String[] parts = code.trim().split("-", 3);
        if (parts.length != 3 || !parts[0].equals("DD1") ||
            !parts[1].matches("[0-9]{1,3}(\\.[0-9]{1,3}){3}") || parts[2].length() != 43) {
            throw new IllegalArgumentException("Paste the full pairing code from the child phone.");
        }
        byte[] key = decode(parts[2]);
        InetAddress address = InetAddress.getByName(parts[1]);
        if (key.length != 32 || !(address instanceof Inet4Address) || !address.isSiteLocalAddress()) {
            throw new IllegalArgumentException("Use the child phone's local pairing code.");
        }
        try (Socket socket = new Socket()) {
            socket.connect(new InetSocketAddress(address, PORT), 3000);
            socket.setSoTimeout(4000);
            DataInputStream in = new DataInputStream(socket.getInputStream());
            DataOutputStream out = new DataOutputStream(socket.getOutputStream());
            byte[] header = new byte[PROTOCOL.length];
            in.readFully(header);
            if (!Arrays.equals(header, PROTOCOL)) throw new IOException("Unsupported link version.");
            byte[] challenge = new byte[16];
            in.readFully(challenge);
            write(out, seal(key, challenge, (byte) 1,
                message.toString().getBytes(StandardCharsets.UTF_8)));
            byte[] response = open(key, challenge, (byte) 2, read(in, MAX_RESPONSE));
            return new JSONObject(new String(response, StandardCharsets.UTF_8));
        }
    }

    private void serve() {
        while (!listener.isClosed()) {
            try (Socket peer = listener.accept()) {
                if (!peer.getInetAddress().isSiteLocalAddress()) continue;
                peer.setSoTimeout(1800);
                exchange(peer);
            } catch (Exception ignored) {
                // A dropped connection or failed authentication affects only this request.
            }
        }
    }

    private void exchange(Socket peer) throws Exception {
        String keyText = state.childKey();
        if (!BlockService.mayShare(state) || keyText.isEmpty()) return;
        byte[] key = decode(keyText);
        byte[] challenge = random(16);
        DataInputStream in = new DataInputStream(peer.getInputStream());
        DataOutputStream out = new DataOutputStream(peer.getOutputStream());
        out.write(PROTOCOL);
        out.write(challenge);
        out.flush();
        byte[] clear = open(key, challenge, (byte) 1, read(in, MAX_REQUEST));
        JSONObject request = new JSONObject(new String(clear, StandardCharsets.UTF_8));
        synchronized (DeviceState.RULE_LOCK) {
            if (listener.isClosed() || !BlockService.mayShare(state) || !keyText.equals(state.childKey())) return;
            if (request.optString("op").equals("block")) {
                if (!(request.opt("blocked") instanceof Boolean) ||
                    !state.changeBlock(request.getString("pkg"), request.getBoolean("blocked"))) {
                    write(out, seal(key, challenge, (byte) 2,
                        "{\"error\":\"This app is unavailable or protected.\"}".getBytes(StandardCharsets.UTF_8)));
                    return;
                }
            } else if (!request.optString("op").equals("read")) {
                return;
            }
        }
        byte[] response = state.snapshot().toString().getBytes(StandardCharsets.UTF_8);
        if (response.length <= MAX_RESPONSE && !listener.isClosed() &&
            BlockService.mayShare(state) && keyText.equals(state.childKey())) {
            write(out, seal(key, challenge, (byte) 2, response));
        }
    }

    private static byte[] seal(byte[] key, byte[] challenge, byte direction, byte[] plain)
        throws GeneralSecurityException {
        byte[] iv = random(12);
        byte[] encrypted = cipher(Cipher.ENCRYPT_MODE, key, challenge, direction, iv).doFinal(plain);
        byte[] frame = Arrays.copyOf(iv, iv.length + encrypted.length);
        System.arraycopy(encrypted, 0, frame, iv.length, encrypted.length);
        return frame;
    }

    private static byte[] open(byte[] key, byte[] challenge, byte direction, byte[] frame)
        throws GeneralSecurityException {
        return cipher(Cipher.DECRYPT_MODE, key, challenge, direction, Arrays.copyOf(frame, 12))
            .doFinal(frame, 12, frame.length - 12);
    }

    private static Cipher cipher(int mode, byte[] key, byte[] challenge, byte direction, byte[] iv)
        throws GeneralSecurityException {
        Cipher cipher = Cipher.getInstance("AES/GCM/NoPadding");
        cipher.init(mode, new SecretKeySpec(key, "AES"), new GCMParameterSpec(128, iv));
        cipher.updateAAD(PROTOCOL);
        cipher.updateAAD(challenge);
        cipher.updateAAD(new byte[]{direction});
        return cipher;
    }

    private static byte[] read(DataInputStream in, int max) throws IOException {
        int length = in.readInt();
        if (length < 28 || length > max + 28) throw new IOException("Invalid family message.");
        byte[] data = new byte[length];
        in.readFully(data);
        return data;
    }

    private static void write(DataOutputStream out, byte[] bytes) throws IOException {
        out.writeInt(bytes.length);
        out.write(bytes);
        out.flush();
    }

    @Override public void close() {
        try {
            listener.close();
        } catch (IOException ignored) {
            // The socket is no longer usable.
        }
    }
}
