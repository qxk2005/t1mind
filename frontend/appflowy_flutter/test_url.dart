import 'package:string_validator/string_validator.dart';

void main() {
  final dataUrl = 'data:image/jpeg;base64,/9j/4AAQSkZJRgABAQEAYABgAAD/2wBDAAMCAgMCAgMDAwMEAwMEBQgFBQQEBQoHBwYIDAoMDAsKCwsNDhIQDQ4RDgsLEBYQERMUFRUVDA8XGBYUGBIUFRT/wB52P8AM1o2tlb2SbYIY4V/2FAoA47Q/A8kjrNqA2R9RCD8x+voK7aONYY1RFCIowFUYAFOooAKKKKACiiigD//2Q==';
  
  print('Testing URL: $dataUrl');
  print('isURL result: ${isURL(dataUrl)}');
  
  // Test with require_protocol option
  print('isURL with require_protocol: ${isURL(dataUrl, {"require_protocol": true})}');
}
