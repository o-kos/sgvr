import wave
import struct
import sys

def get_sample_value_native(wav_file_path, sample_index):
    """
    Считывает WAV-файл и возвращает значение указанного сэмпла,
    используя только встроенные библиотеки Python.

    Args:
        wav_file_path (str): Путь к WAV-файлу.
        sample_index (int): Номер сэмпла (кадра) для извлечения.

    Returns:
        int, tuple или None: Значение сэмпла. Для стерео это будет кортеж.
                              Возвращает None в случае ошибки.
    """
    try:
        with wave.open(wav_file_path, 'rb') as wav_file:
            # Получаем параметры WAV файла
            num_channels = wav_file.getnchannels()
            sample_width = wav_file.getsampwidth()  # Ширина сэмпла в байтах (1, 2, 3, 4)
            num_frames = wav_file.getnframes()
            frame_rate = wav_file.getframerate()

            print(f"Частота дискретизации: {frame_rate} Гц")
            print(f"Количество каналов: {num_channels}")
            print(f"Разрядность: {sample_width * 8} бит")
            print(f"Общее количество сэмплов (кадров): {num_frames}")

            # Проверяем, что запрошенный сэмпл существует
            if not (0 <= sample_index < num_frames):
                print(f"Ошибка: Номер сэмпла ({sample_index}) выходит за пределы диапазона (0-{num_frames - 1})")
                return None

            # Устанавливаем позицию для чтения нужного сэмпла (кадра)
            wav_file.setpos(sample_index)

            # Читаем один кадр (сэмпл для всех каналов)
            frame_data = wav_file.readframes(1)

            # Распаковываем двоичные данные в числовое значение
            if sample_width == 1:  # 8-bit PCM
                # Формат 'B' - беззнаковый char (1 байт)
                format_char = 'B' * num_channels
            elif sample_width == 2:  # 16-bit PCM
                # Формат 'h' - signed short (2 байта), '<' означает little-endian
                format_char = '<' + 'h' * num_channels
            elif sample_width == 4: # 32-bit PCM
                # Формат 'i' - signed int (4 байта)
                format_char = '<' + 'i' * num_channels
            else:
                # 24-битные и другие форматы требуют более сложной обработки
                print(f"Ошибка: Неподдерживаемая разрядность ({sample_width * 8} бит). Поддерживаются 8, 16 и 32 бита.")
                return None

            # Распаковываем кадр
            values = struct.unpack(format_char, frame_data)

            # Если канал один, возвращаем одно число, иначе - кортеж
            return values[0] if num_channels == 1 else values

    except FileNotFoundError:
        print(f"Ошибка: Файл не найден по пути: {wav_file_path}")
        return None
    except wave.Error as e:
        print(f"Ошибка чтения WAV файла: {e}")
        return None
    except Exception as e:
        print(f"Произошла непредвиденная ошибка: {e}")
        return None

if __name__ == "__main__":
    if len(sys.argv) != 3:
        print("Использование: python get_wav_sample.py <путь_к_wav_файлу> <номер_сэмпла>")
        sys.exit(1)

    wav_file = sys.argv[1]
    try:
        sample_num = int(sys.argv[2])
    except ValueError:
        print("Ошибка: Номер сэмпла должен быть целым числом.")
        sys.exit(1)

    value = get_sample_value_native(wav_file, sample_num)

    if value is not None:
        if isinstance(value, tuple):
             print(f"\nЗначение сэмпла #{sample_num}: {value} (Левый канал, Правый канал)")
        else:
             print(f"\nЗначение сэмпла #{sample_num}: {value}")
