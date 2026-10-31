"""
pessoal/Decorador.py - Decoradores e utilitários de telemetria do gTray.

Fornece decoradores para monitoramento de desempenho e tempo de execução
de chamadas aos provedores de inteligência artificial e handlers de eventos.
"""

import time
from functools import wraps
from pessoal.Escriba import registrador

r = registrador()
logger = r.get_logger()

def tempo_decorrido(funcao):
    """
    Decorador que mede e registra no log o tempo total de execução de uma função.
    
    Útil para acompanhar a latência de chamadas à API de IA e o tempo de resposta
    das interações de teclado.
    """
    @wraps(funcao)
    def wrapper(*args, **kwargs):
        inicio = time.time()
        resultado = funcao(*args, **kwargs)
        fim = time.time()
        logger.info(f'{funcao.__name__} levou {fim - inicio:.5f} segundos para executar.')
        return resultado
    return wrapper
