#!/bin/bash

# Создаем директорию для логов, если она не существует
mkdir -p logs

# Получаем текущую дату и время
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")

# Запускаем anchor test и перенаправляем вывод в файл
anchor test 2>&1 | tee "logs/anchor_test_$TIMESTAMP.log"

# Выводим сообщение с информацией о расположении лог-файла
echo "Test results have been logged to logs/anchor_test_$TIMESTAMP.log"